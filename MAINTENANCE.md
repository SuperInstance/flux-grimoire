# MAINTENANCE.md

## Running Tests

```bash
cargo test
```

## Architecture

- `spell.rs` — Spell struct, PatternType enum
- `grimoire.rs` — Grimoire collection with CRUD, search, stats
- `curriculum.rs` — Level-based learning plan with progress tracking
- `lib.rs` — Re-exports

## Considerations

- Spell IDs must be unique within a Grimoire (inscribe deduplicates)
- `prune` removes spells below min_rate after min_uses threshold
- `confidence` = `success_rate` = successes/uses (defaults to 1.0 when unused)
- `import` skips duplicates, returns count of newly added
- Curriculum progress requires all spells in a level to meet min_confidence on average
