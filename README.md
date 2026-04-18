# flux-grimoire

> Pattern library for agent experience accumulation — reusable "spells" with confidence tracking and curriculum-based learning.

## What This Is

`flux-grimoire` is a Rust crate providing a **spell book** for FLUX agents — a structured library of reusable behavioral, cognitive, social, debugging, and optimization patterns ("spells") with confidence tracking, outcome recording, search, and curriculum-based learning with confidence gates.

## Role in the FLUX Ecosystem

The grimoire is how fleet knowledge persists and propagates:

- **`flux-necropolis`** harvests lessons from dead vessels; grimoire teaches them to the living
- **`flux-evolve`** discovers successful mutations; grimoire codifies them as spells
- **`flux-trust`** identifies reliable agents; grimoire captures their strategies
- **`flux-social`** defines mentor relationships; grimoire is the curriculum
- **`flux-dream-cycle`** schedules learning tasks from the grimoire curriculum
- **`flux-skills`** defines runnable skills; grimoire defines the meta-patterns behind them

## Core Concepts

| Concept | Description |
|---------|-------------|
| **Spell** | A reusable pattern with trigger, action, and outcome tracking |
| **PatternType** | Behavioral, Cognitive, Social, Debugging, Optimization |
| **Grimoire** | Collection of spells with search, prune, publish/import |
| **Curriculum** | Ordered study plan with confidence gates (Novice → Adept → Master) |

## Key Features

| Feature | Description |
|---------|-------------|
| **Spell Learning** | `learn()` records a spell with trigger/action/outcome metadata |
| **Outcome Tracking** | `record_outcome()` updates confidence on success/failure |
| **Confidence Scoring** | Automatic confidence calculation from success rate |
| **Search & Prune** | Find spells by name/pattern; remove low-confidence spells |
| **Curriculum System** | 3-level learning path with confidence gates |
| **Publish/Import** | Share spells between grimoires (inter-agent knowledge transfer) |

## Quick Start

```rust
use flux_grimoire::{spell::{Spell, PatternType}, grimoire::Grimoire, curriculum::{Curriculum, Level}};

let mut g = Grimoire::new();

// Learn a spell from experience
g.learn("panic-1", "Don't panic", PatternType::Behavioral,
        "error", "breathe", "always", "me");

// Record outcome to build confidence
g.record_outcome("panic-1", true);  // success
assert_eq!(g.find("panic-1").unwrap().confidence(), 1.0);

// Build curriculum
let mut curriculum = Curriculum::new();
curriculum.add("panic-1", Level::Novice);
curriculum.add("async-debug", Level::Adept);
curriculum.add("fleet-coord", Level::Master);
```

## Building & Testing

```bash
cargo build
cargo test
```

## Related Fleet Repos

- [`flux-necropolis`](https://github.com/SuperInstance/flux-necropolis) — Posthumous lesson harvesting
- [`flux-evolve`](https://github.com/SuperInstance/flux-evolve) — Behavioral evolution
- [`flux-trust`](https://github.com/SuperInstance/flux-trust) — Trust-based spell validation
- [`flux-social`](https://github.com/SuperInstance/flux-social) — Mentorship relationships
- [`flux-skills`](https://github.com/SuperInstance/flux-skills) — Runnable FLUX skills
- [`flux-dream-cycle`](https://github.com/SuperInstance/flux-dream-cycle) — Learning task scheduling

## License

Part of the [SuperInstance](https://github.com/SuperInstance) FLUX fleet.
