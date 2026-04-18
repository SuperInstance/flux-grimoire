use crate::grimoire::Grimoire;

#[derive(Debug, Clone)]
pub struct Level {
    pub name: String,
    pub spell_ids: Vec<String>,
    pub min_confidence: f64,
}

#[derive(Debug, Clone)]
pub struct Curriculum {
    pub levels: Vec<Level>,
}

#[derive(Debug, Clone)]
pub struct Progress {
    pub mastered_levels: usize,
    pub total_levels: usize,
    pub details: Vec<bool>,
}

impl Curriculum {
    pub fn new(levels: Vec<Level>) -> Self {
        Self { levels }
    }

    pub fn progress(&self, grimoire: &Grimoire) -> Progress {
        let total = self.levels.len();
        let details: Vec<bool> = self
            .levels
            .iter()
            .map(|level| {
                let mut count = 0u32;
                let mut total_conf = 0.0;
                for id in &level.spell_ids {
                    if let Some(spell) = grimoire.find(id) {
                        count += 1;
                        total_conf += spell.confidence();
                    }
                }
                if count == 0 {
                    return false;
                }
                let avg = total_conf / count as f64;
                avg >= level.min_confidence
            })
            .collect();
        let mastered = details.iter().filter(|&&x| x).count();
        Progress {
            mastered_levels: mastered,
            total_levels: total,
            details,
        }
    }
}
