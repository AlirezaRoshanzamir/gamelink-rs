# Gamelink

A Rust framework for implementing and simulating strategy/board games with pluggable AI agents.

## Overview

`gamelink` provides a set of traits and utilities for building turn-based games that support:

- **Reversible actions** — steps can be undone via `apply`/`revert`, enabling tree search over game states
- **Pluggable decision strategies** — swap between random sampling, human CLI, and AI players at runtime
- **Probabilistic decisions** — actions are wrapped in `Probabilistic<T>` with weights, supporting both uniform and custom distributions
- **Simulation helper** — temporarily replace players for lookahead or AI evaluation, with automatic rollback on drop

## Requirements

- Rust 1.85+ (edition 2024)
- Cargo

## Getting Started

```bash
git clone <repo-url>
cd gamelink
cargo run
```

By default this runs a Tic-Tac-Toe (XO) game in CLI mode, prompting each player to select a cell on their turn.

## Architecture

### Core traits (`src/game.rs`)

| Trait | Responsibility |
|-------|---------------|
| `Game` | Owns players, drives the game loop (`step_forward` / `step_backward`), reports termination and scores |
| `Action` | Encapsulates a single move — `is_feasible`, `apply`, `revert` |
| `Player` | Picks an action given the current game state |
| `DecisionSelector` | Strategy for choosing among a weighted list of options |

`DecisionSelectorExtension` is a blanket-impl'd trait that adds the ergonomic `select` / `select_index` helpers on top of the raw `select_index_core` primitive.

### Built-in selectors

| Selector | Behaviour |
|----------|-----------|
| `CliDecisionSelector` | Prompts the human via stdin |
| `SamplingDecisionSelector` | Samples randomly according to the given probability weights |

### `Probabilistic<T>` (`src/probabilistic.rs`)

A thin wrapper that pairs any event with a probability in `[0, 1]`. Convenience constructors:

- `Probabilistic::deterministic(event)` — probability 1.0
- `Probabilistic::many_uniform(events)` — equal weight for each event
- `Probabilistic::many_from_mapping(iter)` — explicit weights

### `Simulation` RAII guard (`src/game.rs`)

Temporarily swaps players in and out of a live game for AI lookahead. Original players are restored automatically when the guard is dropped.

## Example game: Tic-Tac-Toe (`src/xo.rs`)

```rust
let decision_selector: Rc<dyn DecisionSelector> = Rc::new(CliDecisionSelector {});

let mut game = XOGame::new();
game.join_player(XOPlayerRole::X, XOPlayer::new(Rc::clone(&decision_selector)));
game.join_player(XOPlayerRole::O, XOPlayer::new(Rc::clone(&decision_selector)));

game.step_all_forward();
```

Switch to random play by replacing `CliDecisionSelector` with `SamplingDecisionSelector`.

## Logging

Gamelink uses the `log` crate. Enable output with the `RUST_LOG` environment variable:

```bash
RUST_LOG=info cargo run   # show each move and board state
RUST_LOG=debug cargo run  # also show simulation steps
```

## License

MIT
