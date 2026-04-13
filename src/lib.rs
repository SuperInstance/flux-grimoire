pub mod curriculum;
pub mod pattern;
pub mod spellbook;
pub mod catalog;

#[cfg(test)]
mod tests {
    use crate::curriculum::{Curriculum, Level};
    use crate::pattern::Pattern;
    use crate::spellbook::SpellBook;
    use crate::catalog::PatternCatalog;

    // --- Spell tests ---
    #[test]
    fn spell_defaults() {
        let s = Spell::new("id1", "Test", PatternType::Behavioral, "t", "a", "c", "me");
        assert_eq!(s.uses, 0);
        assert_eq!(s.successes, 0);
        assert_eq!(s.failures, 0);
        assert!(!s.shared);
        assert_eq!(s.id, "id1");
    }

    #[test]
    fn success_rate_default() {
        let s = Spell::new("id1", "Test", PatternType::Behavioral, "t", "a", "c", "me");
        assert_eq!(s.success_rate(), 1.0);
    }

    #[test]
    fn success_rate_computed() {
        let mut s = Spell::new("id1", "Test", PatternType::Behavioral, "t", "a", "c", "me");
        s.record_use(true);
        s.record_use(true);
        s.record_use(false);
        assert!((s.success_rate() - 2.0 / 3.0).abs() < 1e-9);
        assert_eq!(s.uses, 3);
    }

    #[test]
    fn confidence_equals_success_rate() {
        let mut s = Spell::new("id1", "Test", PatternType::Cognitive, "t", "a", "c", "me");
        s.record_use(true);
        s.record_use(false);
        assert_eq!(s.confidence(), s.success_rate());
    }

    #[test]
    fn should_forget_false_unused() {
        let s = Spell::new("id1", "Test", PatternType::Behavioral, "t", "a", "c", "me");
        assert!(!s.should_forget(0.5, 3));
    }

    #[test]
    fn should_forget_true() {
        let mut s = Spell::new("id1", "Test", PatternType::Behavioral, "t", "a", "c", "me");
        s.record_use(false);
        s.record_use(false);
        s.record_use(false);
        assert!(s.should_forget(0.5, 3));
    }

    // --- Grimoire tests ---
    fn make_grimoire() -> Grimoire {
        let mut g = Grimoire::new();
        g.learn(
            "s1",
            "Debug Print",
            PatternType::Debugging,
            "error",
            "println",
            "dev",
            "agent",
        );
        g.learn(
            "s2",
            "Deep Breath",
            PatternType::Behavioral,
            "panic",
            "breathe",
            "always",
            "agent",
        );
        g.learn(
            "s3",
            "Refactor",
            PatternType::Optimization,
            "slow",
            "profile",
            "perf",
            "agent",
        );
        g
    }

    #[test]
    fn grimoire_inscribe_and_find() {
        let mut g = Grimoire::new();
        let s = Spell::new("x", "X", PatternType::Social, "t", "a", "c", "me");
        assert!(g.inscribe(s.clone()));
        assert!(!g.inscribe(s)); // duplicate
        assert!(g.find("x").is_some());
    }

    #[test]
    fn grimoire_cast() {
        let g = make_grimoire();
        let results = g.cast("error");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "s1");
    }

    #[test]
    fn grimoire_record_outcome() {
        let mut g = make_grimoire();
        assert!(g.record_outcome("s1", true));
        assert!(!g.record_outcome("nonexistent", true));
        assert_eq!(g.find("s1").unwrap().successes, 1);
    }

    #[test]
    fn grimoire_search_trigger() {
        let g = make_grimoire();
        assert_eq!(g.search_trigger("err").len(), 1);
    }

    #[test]
    fn grimoire_by_type() {
        let g = make_grimoire();
        assert_eq!(g.by_type(&PatternType::Debugging).len(), 1);
        assert_eq!(g.by_type(&PatternType::Social).len(), 0);
    }

    #[test]
    fn grimoire_by_confidence() {
        let mut g = make_grimoire();
        g.record_outcome("s1", false);
        assert_eq!(g.by_confidence(1.0).len(), 2); // unused ones have 1.0
    }

    #[test]
    fn grimoire_prune() {
        let mut g = make_grimoire();
        for _ in 0..5 {
            g.record_outcome("s1", false);
        }
        let removed = g.prune(0.5, 5);
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].id, "s1");
        assert_eq!(g.find("s1"), None);
    }

    #[test]
    fn grimoire_publish_and_shared() {
        let mut g = make_grimoire();
        assert!(g.publish("s1"));
        assert!(!g.publish("nope"));
        assert_eq!(g.shared().len(), 1);
    }

    #[test]
    fn grimoire_import() {
        let mut g = make_grimoire();
        let new_spell = Spell::new("s_new", "New", PatternType::Social, "t", "a", "c", "me");
        let dup_spell = Spell::new("s1", "Dup", PatternType::Social, "t", "a", "c", "me");
        assert_eq!(g.import(vec![new_spell, dup_spell]), 1);
        assert_eq!(g.find("s_new").unwrap().name, "New");
    }

    #[test]
    fn grimoire_statistics() {
        let mut g = make_grimoire();
        g.publish("s1");
        let stats = g.statistics();
        assert_eq!(stats.total, 3);
        assert_eq!(stats.shared, 1);
        assert_eq!(stats.by_type.len(), 3);
        assert!(stats.avg_confidence > 0.0);
    }

    // --- Curriculum tests ---
    #[test]
    fn curriculum_progress_empty() {
        let c = Curriculum::new(vec![Level {
            name: "L1".into(),
            spell_ids: vec!["s1".into()],
            min_confidence: 0.8,
        }]);
        let g = Grimoire::new();
        let p = c.progress(&g);
        assert_eq!(p.mastered_levels, 0);
        assert_eq!(p.total_levels, 1);
    }

    #[test]
    fn curriculum_progress_mastered() {
        let mut g = make_grimoire();
        for _ in 0..10 {
            g.record_outcome("s1", true);
        } // confidence = 1.0
        let c = Curriculum::new(vec![
            Level {
                name: "L1".into(),
                spell_ids: vec!["s1".into()],
                min_confidence: 0.9,
            },
            Level {
                name: "L2".into(),
                spell_ids: vec!["s2".into()],
                min_confidence: 0.8,
            },
        ]);
        let p = c.progress(&g);
        assert_eq!(p.mastered_levels, 2); // s1 used=1.0, s2 unused=1.0 both pass
    }

    // --- Pattern + SpellBook integration tests ---
    #[test]
    fn spellbook_compose_chain() {
        let mut sb = SpellBook::new("Math", "superz");
        sb.add_pattern(Pattern::new("load", "Load", &[0x10]));
        sb.add_pattern(Pattern::new("add", "Add", &[0x20]).with_dependencies(&["load"]));
        sb.add_pattern(Pattern::new("store", "Store", &[0x30]).with_dependencies(&["add"]));
        // Compose all three with dependency order
        let result = sb.compose(&["store", "add", "load"]);
        assert!(result.is_ok());
        let bc = result.unwrap();
        assert_eq!(bc, vec![0x10, 0x20, 0x30]); // load -> add -> store
    }

    #[test]
    fn catalog_and_spellbook_integration() {
        let mut cat = PatternCatalog::new();
        let p1 = Pattern::new("op_add", "Add", &[0x01, 0x02]).with_category("arithmetic").with_tags(&["math"]);
        let p2 = Pattern::new("op_mul", "Multiply", &[0x03, 0x04]).with_category("arithmetic").with_tags(&["math", "fast"]);
        cat.register(p1.clone());
        cat.register(p2.clone());

        let mut sb = SpellBook::new("MathOps", "superz");
        for id in cat.search("arithmetic") {
            if let Some(pat) = cat.get(&id.pattern_id) {
                sb.add_pattern(pat.clone());
            }
        }
        assert_eq!(sb.len(), 2);
    }

    #[test]
    fn version_tracking_across_updates() {
        let mut sb = SpellBook::new("Versioned", "superz");
        sb.add_pattern(Pattern::new("p1", "Pattern", &[0x01, 0x02]));
        {
            let p = sb.get_pattern_mut("p1").unwrap();
            assert_eq!(p.version, 1);
            p.bump_version("first update");
        }
        {
            let p = sb.get_pattern_mut("p1").unwrap();
            assert_eq!(p.version, 2);
            p.update_bytecode(&[0xFF, 0xEE]);
        }
        assert_eq!(sb.get_pattern("p1").unwrap().version, 3);
    }

    #[test]
    fn export_import_preserves_data() {
        let mut sb = SpellBook::new("ExportTest", "superz")
            .with_description("Test export/import");
        let mut p = Pattern::new("p1", "Complex", &[0x01, 0x02])
            .with_category("math")
            .with_tags(&["basic", "essential"]);
        p.bump_version("initial release");
        sb.add_pattern(p);

        let exported = sb.export();
        let imported = SpellBook::from_export(&exported);
        assert_eq!(imported.name, "ExportTest");
        assert_eq!(imported.description, "Test export/import");
        assert_eq!(imported.get_pattern("p1").unwrap().version, 2);
        assert_eq!(imported.get_pattern("p1").unwrap().tags, vec!["basic", "essential"]);
    }
}
