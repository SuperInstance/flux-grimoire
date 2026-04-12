pub mod curriculum;
pub mod grimoire;
pub mod spell;

#[cfg(test)]
mod tests {
    use crate::curriculum::{Curriculum, Level};
    use crate::grimoire::Grimoire;
    use crate::spell::{PatternType, Spell};

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

    // --- Additional edge-case tests ---

    #[test]
    fn pattern_type_display() {
        assert_eq!(format!("{}", PatternType::Behavioral), "Behavioral");
        assert_eq!(format!("{}", PatternType::Cognitive), "Cognitive");
        assert_eq!(format!("{}", PatternType::Social), "Social");
        assert_eq!(format!("{}", PatternType::Debugging), "Debugging");
        assert_eq!(format!("{}", PatternType::Optimization), "Optimization");
    }

    #[test]
    fn empty_grimoire_operations() {
        let mut g = Grimoire::new();
        assert!(g.find("nothing").is_none());
        assert!(g.cast("none").is_empty());
        assert!(g.search_trigger("x").is_empty());
        assert!(g.by_type(&PatternType::Behavioral).is_empty());
        assert!(g.by_confidence(0.0).is_empty());
        assert!(g.shared().is_empty());
        assert!(!g.publish("nope"));
        assert!(!g.record_outcome("nope", true));
        assert_eq!(g.import(vec![]), 0);
        let stats = g.statistics();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.shared, 0);
        assert!(stats.by_type.is_empty());
        assert_eq!(stats.avg_confidence, 0.0);
    }

    #[test]
    fn grimoire_prune_nothing_removed() {
        let mut g = make_grimoire();
        let removed = g.prune(0.0, 100);
        assert!(removed.is_empty());
        assert_eq!(g.find("s1").unwrap().id, "s1");
    }

    #[test]
    fn grimoire_import_empty_vec() {
        let mut g = make_grimoire();
        assert_eq!(g.import(vec![]), 0);
    }

    #[test]
    fn grimoire_cast_multiple_same_trigger() {
        let mut g = Grimoire::new();
        g.learn(
            "a",
            "Spell A",
            PatternType::Behavioral,
            "fire",
            "run",
            "always",
            "me",
        );
        g.learn(
            "b",
            "Spell B",
            PatternType::Cognitive,
            "fire",
            "think",
            "always",
            "me",
        );
        let results = g.cast("fire");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn spell_all_fields() {
        let s = Spell::new(
            "id",
            "Name",
            PatternType::Social,
            "trig",
            "act",
            "ctx",
            "auth",
        );
        assert_eq!(s.id, "id");
        assert_eq!(s.name, "Name");
        assert_eq!(s.pattern_type, PatternType::Social);
        assert_eq!(s.trigger, "trig");
        assert_eq!(s.action, "act");
        assert_eq!(s.context, "ctx");
        assert_eq!(s.author, "auth");
        assert_eq!(s.uses, 0);
        assert_eq!(s.successes, 0);
        assert_eq!(s.failures, 0);
        assert!(!s.shared);
    }

    #[test]
    fn curriculum_empty_levels() {
        let c = Curriculum::new(vec![]);
        let g = Grimoire::new();
        let p = c.progress(&g);
        assert_eq!(p.mastered_levels, 0);
        assert_eq!(p.total_levels, 0);
        assert!(p.details.is_empty());
    }

    #[test]
    fn curriculum_level_missing_spell() {
        let c = Curriculum::new(vec![Level {
            name: "L1".into(),
            spell_ids: vec!["nonexistent".into()],
            min_confidence: 0.5,
        }]);
        let g = Grimoire::new();
        let p = c.progress(&g);
        assert_eq!(p.mastered_levels, 0);
        assert!(!p.details[0]);
    }

    #[test]
    fn grimoire_search_trigger_no_match() {
        let g = make_grimoire();
        assert!(g.search_trigger("zzzzz").is_empty());
    }

    #[test]
    fn grimoire_publish_twice() {
        let mut g = make_grimoire();
        assert!(g.publish("s1"));
        assert!(g.publish("s1")); // second publish should still succeed
        assert_eq!(g.shared().len(), 1);
    }

    #[test]
    fn grimoire_record_outcome_failure() {
        let mut g = make_grimoire();
        g.record_outcome("s1", false);
        let s = g.find("s1").unwrap();
        assert_eq!(s.uses, 1);
        assert_eq!(s.successes, 0);
        assert_eq!(s.failures, 1);
    }
}
