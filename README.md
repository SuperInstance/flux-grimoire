# flux-grimoire

Pattern library for agent experience accumulation. Agents accumulate reusable "spells" — behavioral, cognitive, social, debugging, and optimization patterns — with confidence tracking and curriculum-based learning.

## Structure

- **Spell** — A reusable pattern with trigger, action, and outcome tracking
- **Grimoire** — Collection of spells with search, prune, publish/import
- **Curriculum** — Ordered study plan with confidence gates

## Quick Start

```rust
use flux_grimoire::{spell::{Spell, PatternType}, grimoire::Grimoire, curriculum::{Curriculum, Level}};

let mut g = Grimoire::new();
g.learn("panic-1", "Don't panic", PatternType::Behavioral, "error", "breathe", "always", "me");
g.record_outcome("panic-1", true);
assert_eq!(g.find("panic-1").unwrap().confidence(), 1.0);
```
