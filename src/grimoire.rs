use crate::spell::{PatternType, Spell};

#[derive(Debug, Clone)]
pub struct Grimoire {
    spells: Vec<Spell>,
}

#[derive(Debug, Clone)]
pub struct GrimoireStats {
    pub total: usize,
    pub shared: usize,
    pub by_type: Vec<(String, usize)>,
    pub avg_confidence: f64,
}

impl Grimoire {
    pub fn new() -> Self {
        Self { spells: vec![] }
    }

    pub fn inscribe(&mut self, spell: Spell) -> bool {
        if self.spells.iter().any(|s| s.id == spell.id) {
            false
        } else {
            self.spells.push(spell);
            true
        }
    }

    pub fn learn(
        &mut self,
        id: &str,
        name: &str,
        pattern_type: PatternType,
        trigger: &str,
        action: &str,
        context: &str,
        author: &str,
    ) -> &Spell {
        let spell = Spell::new(id, name, pattern_type, trigger, action, context, author);
        self.spells.push(spell);
        self.spells.last().unwrap()
    }

    pub fn cast(&self, trigger: &str) -> Vec<&Spell> {
        self.spells
            .iter()
            .filter(|s| s.trigger == trigger)
            .collect()
    }

    pub fn record_outcome(&mut self, id: &str, success: bool) -> bool {
        if let Some(spell) = self.spells.iter_mut().find(|s| s.id == id) {
            spell.record_use(success);
            true
        } else {
            false
        }
    }

    pub fn find(&self, id: &str) -> Option<&Spell> {
        self.spells.iter().find(|s| s.id == id)
    }

    pub fn search_trigger(&self, query: &str) -> Vec<&Spell> {
        self.spells
            .iter()
            .filter(|s| s.trigger.contains(query))
            .collect()
    }

    pub fn by_type(&self, pattern_type: &PatternType) -> Vec<&Spell> {
        self.spells
            .iter()
            .filter(|s| s.pattern_type == *pattern_type)
            .collect()
    }

    pub fn by_confidence(&self, min: f64) -> Vec<&Spell> {
        self.spells
            .iter()
            .filter(|s| s.confidence() >= min)
            .collect()
    }

    pub fn prune(&mut self, min_rate: f64, min_uses: u32) -> Vec<Spell> {
        let (keep, remove): (Vec<_>, Vec<_>) = self
            .spells
            .drain(..)
            .partition(|s| !s.should_forget(min_rate, min_uses));
        self.spells = keep;
        remove
    }

    pub fn publish(&mut self, id: &str) -> bool {
        if let Some(spell) = self.spells.iter_mut().find(|s| s.id == id) {
            spell.shared = true;
            true
        } else {
            false
        }
    }

    pub fn shared(&self) -> Vec<&Spell> {
        self.spells.iter().filter(|s| s.shared).collect()
    }

    pub fn import(&mut self, spells: Vec<Spell>) -> usize {
        let mut count = 0;
        for spell in spells {
            if self.inscribe(spell) {
                count += 1;
            }
        }
        count
    }

    pub fn statistics(&self) -> GrimoireStats {
        let total = self.spells.len();
        let shared = self.spells.iter().filter(|s| s.shared).count();
        let mut type_counts = std::collections::HashMap::new();
        for s in &self.spells {
            *type_counts
                .entry(s.pattern_type.to_string())
                .or_insert(0usize) += 1;
        }
        let mut by_type: Vec<_> = type_counts.into_iter().collect();
        by_type.sort_by(|a, b| a.0.cmp(&b.0));
        let avg_confidence = if total > 0 {
            self.spells.iter().map(|s| s.confidence()).sum::<f64>() / total as f64
        } else {
            0.0
        };
        GrimoireStats {
            total,
            shared,
            by_type,
            avg_confidence,
        }
    }
}
