use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatternType {
    Behavioral,
    Cognitive,
    Social,
    Debugging,
    Optimization,
}

impl fmt::Display for PatternType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spell {
    pub id: String,
    pub name: String,
    pub pattern_type: PatternType,
    pub trigger: String,
    pub action: String,
    pub context: String,
    pub author: String,
    pub uses: u32,
    pub successes: u32,
    pub failures: u32,
    pub shared: bool,
}

impl Spell {
    pub fn new(id: &str, name: &str, pattern_type: PatternType, trigger: &str, action: &str, context: &str, author: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            pattern_type,
            trigger: trigger.to_string(),
            action: action.to_string(),
            context: context.to_string(),
            author: author.to_string(),
            uses: 0,
            successes: 0,
            failures: 0,
            shared: false,
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.uses == 0 { return 1.0; }
        self.successes as f64 / self.uses as f64
    }

    pub fn confidence(&self) -> f64 {
        self.success_rate()
    }

    pub fn record_use(&mut self, success: bool) {
        self.uses += 1;
        if success { self.successes += 1; } else { self.failures += 1; }
    }

    pub fn should_forget(&self, min_rate: f64, min_uses: u32) -> bool {
        self.uses >= min_uses && self.success_rate() < min_rate
    }
}
