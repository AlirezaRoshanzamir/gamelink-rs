use crate::probabilistic::Probabilistic;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::{self, Write};

use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;

pub trait DecisionSelector {
    fn select_index_core(
        &self,
        probabilities: &[f64],
        labels: &[&str],
        title: Option<&str>,
    ) -> usize;
}

pub trait DecisionSelectorExtension: DecisionSelector {
    fn select<'a, TDecision>(
        &self,
        decisions: impl IntoIterator<Item = Probabilistic<TDecision>>
        + AsRef<[Probabilistic<TDecision>]>,
        title: impl Into<Option<&'a str>>,
    ) -> TDecision
    where
        TDecision: std::fmt::Display,
    {
        let index = self.select_index(decisions.as_ref(), title);
        decisions.into_iter().nth(index).unwrap().event
    }

    fn select_index<'a, TDecision>(
        &self,
        decisions: &[Probabilistic<TDecision>],
        title: impl Into<Option<&'a str>>,
    ) -> usize
    where
        TDecision: std::fmt::Display,
    {
        let labels: Vec<String> = decisions.iter().map(|d| format!("{}", d.event)).collect();
        let label_refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();
        self.select_index_core(
            &decisions.iter().map(|d| d.probability).collect::<Vec<_>>(),
            &label_refs,
            title.into(),
        )
    }
}

impl<TDecisionSelector: DecisionSelector + ?Sized> DecisionSelectorExtension for TDecisionSelector {}

impl<TDecisionSelector: DecisionSelector> DecisionSelector for Box<TDecisionSelector> {
    fn select_index_core(
        &self,
        probabilities: &[f64],
        labels: &[&str],
        title: Option<&str>,
    ) -> usize {
        (**self).select_index_core(probabilities, labels, title)
    }
}

pub struct SamplingDecisionSelector;

impl DecisionSelector for SamplingDecisionSelector {
    fn select_index_core(
        &self,
        probabilities: &[f64],
        _labels: &[&str],
        _title: Option<&str>,
    ) -> usize {
        let dist = WeightedIndex::new(probabilities).expect("Invalid weights");
        let mut rng = rand::rng();
        dist.sample(&mut rng)
    }
}

pub struct CliDecisionSelector;

impl DecisionSelector for CliDecisionSelector {
    fn select_index_core(
        &self,
        _probabilities: &[f64],
        labels: &[&str],
        title: Option<&str>,
    ) -> usize {
        let decisions_part: String = labels
            .iter()
            .enumerate()
            .map(|(i, label)| format!("{}: {}", i + 1, label))
            .collect::<Vec<_>>()
            .join(", ");

        let title_part = title.map_or(String::new(), |t| format!("\"{t}\" "));

        loop {
            print!("Select {title_part}between ({decisions_part}): ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            if let Ok(n) = input.trim().parse::<usize>() {
                if n >= 1 && n <= labels.len() {
                    return n - 1;
                }
            }
        }
    }
}

pub trait Action {
    type Game: Game<Action = Self>;

    fn is_feasible(&self, game: &Self::Game) -> bool;
    fn apply(&self, game: &mut Self::Game);
    fn revert(&self, game: &mut Self::Game);
}

pub trait Player {
    type Game: Game<Player = Self>;

    fn act(&self, state: &<Self::Game as Game>::State) -> <Self::Game as Game>::Action;
}

pub trait Game {
    type Player: Player;
    type Role: Eq + Hash + Clone;
    type State;
    type Action: Action;

    fn join_player(&mut self, role: Self::Role, player: Self::Player);
    fn replace_player(&mut self, role: Self::Role, new: Self::Player) -> Self::Player;
    fn get_each_role_player(&self) -> &HashMap<Self::Role, Self::Player>;
    fn finished(&self) -> bool;
    fn possible_actions_for_player(
        &self,
        player: &Self::Player,
        state: &Self::State,
    ) -> impl IntoIterator<Item = Self::Action>;
    fn final_score_for_player(&self, role: Self::Role) -> f64;
    fn step_forward(&mut self);
    fn step_backward(&mut self);

    fn step_all_forward(&mut self) {
        while !self.finished() {
            self.step_forward();
        }
    }

    fn log(&self, message: &str) {
        log::info!("{}", message);
    }
}

// ─── Simulation helper ──────────────────────────────────────────────────────

pub struct Simulation<'a, TGame: Game> {
    game: &'a mut TGame,
    replaced: HashMap<TGame::Role, TGame::Player>,
}

impl<'a, TGame: Game> Simulation<'a, TGame> {
    pub fn begin(game: &'a mut TGame, replacements: HashMap<TGame::Role, TGame::Player>) -> Self {
        let mut replaced: HashMap<TGame::Role, TGame::Player> = HashMap::new();
        for (role, new_player) in replacements.into_iter() {
            let old_player = game.replace_player(role.clone(), new_player);
            replaced.insert(role, old_player);
        }
        Simulation { game, replaced }
    }
}

impl<'a, TGame: Game> Drop for Simulation<'a, TGame> {
    fn drop(&mut self) {
        for (role, old_player) in self.replaced.drain() {
            self.game.replace_player(role, old_player);
        }
    }
}

impl<'a, TGame: Game> Game for Simulation<'a, TGame> {
    type Player = TGame::Player;
    type Role = TGame::Role;
    type State = TGame::State;
    type Action = TGame::Action;

    fn join_player(&mut self, role: Self::Role, player: Self::Player) {
        self.game.join_player(role, player);
    }

    fn replace_player(&mut self, role: Self::Role, new: Self::Player) -> Self::Player {
        self.game.replace_player(role, new)
    }

    fn get_each_role_player(&self) -> &HashMap<Self::Role, Self::Player> {
        self.game.get_each_role_player()
    }

    fn finished(&self) -> bool {
        self.game.finished()
    }

    fn possible_actions_for_player(
        &self,
        player: &Self::Player,
        state: &Self::State,
    ) -> impl IntoIterator<Item = Self::Action> {
        self.game.possible_actions_for_player(player, state)
    }

    fn final_score_for_player(&self, role: Self::Role) -> f64 {
        self.game.final_score_for_player(role)
    }

    fn step_forward(&mut self) {
        self.game.step_forward();
    }

    fn step_backward(&mut self) {
        self.game.step_backward();
    }

    fn log(&self, message: &str) {
        log::debug!("{}", message);
    }
}
