use crate::game::{
    Action, DecisionSelector, DecisionSelectorExtension, Game, Player,
};
use crate::probabilistic::Probabilistic;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum XOPlayerRole {
    X,
    O,
}

impl XOPlayerRole {
    pub fn opposite(self) -> Self {
        match self {
            XOPlayerRole::X => XOPlayerRole::O,
            XOPlayerRole::O => XOPlayerRole::X,
        }
    }
}

impl fmt::Display for XOPlayerRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XOPlayerRole::X => write!(f, "X"),
            XOPlayerRole::O => write!(f, "O"),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Select {
    pub row: usize,
    pub col: usize,
}

impl Select {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

impl fmt::Display for Select {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Select({}, {})", self.row, self.col)
    }
}

impl Action for Select {
    type Game = XOGame;

    fn is_feasible(&self, game: &Self::Game) -> bool {
        game.board[self.row][self.col].is_none()
    }

    fn apply(&self, game: &mut Self::Game) {
        game.board[self.row][self.col] = Some(game.turn);
        game.turn = game.turn.opposite();
    }

    fn revert(&self, game: &mut Self::Game) {
        game.board[self.row][self.col] = None;
        game.turn = game.turn.opposite();
    }
}

pub struct XOPlayer {
    decision_selector: Rc<dyn DecisionSelector>,
}

impl XOPlayer {
    pub fn new(decision_selector: Rc<dyn DecisionSelector>) -> Self {
        Self { decision_selector }
    }
}

impl Player for XOPlayer {
    type Game = XOGame;

    fn act(&self, state: &<XOGame as Game>::State) -> <Self::Game as Game>::Action {
        let actions = state.possible_actions_for_player(self, state);
        let decisions = Probabilistic::many_uniform(actions);
        self.decision_selector.select(decisions, "Cell Selection")
    }
}

pub const BOARD_SIZE: usize = 3;

pub struct XOGame {
    pub board: [[Option<<XOGame as Game>::Role>; BOARD_SIZE]; BOARD_SIZE],
    pub turn: <XOGame as Game>::Role,
    role_to_player: HashMap<<XOGame as Game>::Role, <XOGame as Game>::Player>,
    history: Vec<<XOGame as Game>::Action>,
}

impl XOGame {
    pub fn new() -> Self {
        Self {
            board: [[None; BOARD_SIZE]; BOARD_SIZE],
            turn: XOPlayerRole::X,
            role_to_player: HashMap::new(),
            history: Vec::new(),
        }
    }

    pub fn winner(&self) -> Option<<XOGame as Game>::Role> {
        let b = &self.board;
        let size = BOARD_SIZE;

        // Check rows
        for r in 0..size {
            if let Some(role) = b[r][0] {
                if b[r].iter().all(|&cell| cell == Some(role)) {
                    return Some(role);
                }
            }
        }

        // Check columns
        for c in 0..size {
            if let Some(role) = b[0][c] {
                if (0..size).all(|r| b[r][c] == Some(role)) {
                    return Some(role);
                }
            }
        }

        // Main diagonal
        if let Some(role) = b[0][0] {
            if (0..size).all(|i| b[i][i] == Some(role)) {
                return Some(role);
            }
        }

        // Anti-diagonal
        if let Some(role) = b[0][size - 1] {
            if (0..size).all(|i| b[i][size - 1 - i] == Some(role)) {
                return Some(role);
            }
        }

        None
    }

    pub fn number_of_filled_cells(&self) -> usize {
        self.board
            .iter()
            .flat_map(|row| row.iter())
            .filter(|cell| cell.is_some())
            .count()
    }
}

impl fmt::Display for XOGame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, row) in self.board.iter().enumerate() {
            if i > 0 {
                writeln!(f, "---------")?;
            }
            let row_str: Vec<&str> = row
                .iter()
                .map(|cell| match cell {
                    Some(XOPlayerRole::X) => "X",
                    Some(XOPlayerRole::O) => "O",
                    None => ".",
                })
                .collect();
            writeln!(f, "{}", row_str.join(" | "))?;
        }
        Ok(())
    }
}

impl Game for XOGame {
    type State = XOGame;
    type Action = Select;
    type Player = XOPlayer;
    type Role = XOPlayerRole;

    fn get_each_role_player(&self) -> &HashMap<Self::Role, Self::Player> {
        &self.role_to_player
    }

    fn finished(&self) -> bool {
        if self.winner().is_some() {
            return true;
        }
        self.board
            .iter()
            .flat_map(|row| row.iter())
            .all(|cell| cell.is_some())
    }

    #[allow(refining_impl_trait)]
    fn possible_actions_for_player(
        &self,
        _player: &Self::Player,
        _state: &Self::State,
    ) -> impl IntoIterator<IntoIter: ExactSizeIterator, Item = Self::Action> {
        let mut actions = Vec::new();
        for row in 0..BOARD_SIZE {
            for col in 0..BOARD_SIZE {
                if self.board[row][col].is_none() {
                    actions.push(Select::new(row, col));
                }
            }
        }
        actions
    }

    fn final_score_for_player(&self, role: Self::Role) -> f64 {
        match self.winner() {
            None => 0.0,
            Some(winner) if winner == role => 1.0,
            Some(_) => -1.0,
        }
    }

    fn step_forward(&mut self) {
        let player = &self.role_to_player[&self.turn];
        let action = player.act(self);
        action.apply(self);

        let winner_str = self.winner().map_or("none".to_string(), |w| w.to_string());
        self.log(&format!(
            "Action: {}, Winner: {}, Game Board:\n{}",
            action, winner_str, self
        ));

        self.history.push(action);
    }

    fn step_backward(&mut self) {
        if let Some(action) = self.history.pop() {
            action.revert(self);
            self.log(&format!(
                "Action reverted: {}, Game Board:\n{}",
                action, self
            ));
        }
    }

    fn join_player(&mut self, role: Self::Role, player: Self::Player) {
        if self.role_to_player.contains_key(&role) {
            panic!("A player with role {} has already joined.", role);
        }
        self.role_to_player.insert(role, player);
    }

    fn replace_player(&mut self, role: Self::Role, new: Self::Player) -> Self::Player {
        self.role_to_player
            .insert(role, new)
            .expect("Role must already have a player.")
    }
}
