use crate::pattern::Pattern;
use std::collections::HashMap;

/// A named collection of reusable FLUX bytecode patterns/recipes.
#[derive(Debug, Clone)]
pub struct SpellBook {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub patterns: HashMap<String, Pattern>,
    pub created_at: u64,
    pub updated_at: u64,
}

impl SpellBook {
    pub fn new(name: &str, author: &str) -> Self {
        let now = Self::now_ms();
        Self {
            name: name.to_string(),
            description: String::new(),
            author: author.to_string(),
            version: "1.0.0".to_string(),
            patterns: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Add a pattern to the spell book.
    pub fn add_pattern(&mut self, pattern: Pattern) -> bool {
        let id = pattern.id.clone();
        if self.patterns.contains_key(&id) {
            false
        } else {
            self.patterns.insert(id, pattern);
            self.updated_at = Self::now_ms();
            true
        }
    }

    /// Add or replace a pattern.
    pub fn set_pattern(&mut self, pattern: Pattern) {
        let id = pattern.id.clone();
        self.patterns.insert(id, pattern);
        self.updated_at = Self::now_ms();
    }

    /// Remove a pattern by ID. Returns the removed pattern.
    pub fn remove_pattern(&mut self, id: &str) -> Option<Pattern> {
        let result = self.patterns.remove(id);
        if result.is_some() {
            self.updated_at = Self::now_ms();
        }
        result
    }

    /// Get a pattern by ID.
    pub fn get_pattern(&self, id: &str) -> Option<&Pattern> {
        self.patterns.get(id)
    }

    /// Get a mutable pattern by ID.
    pub fn get_pattern_mut(&mut self, id: &str) -> Option<&mut Pattern> {
        self.patterns.get_mut(id)
    }

    /// List all pattern IDs.
    pub fn pattern_ids(&self) -> Vec<&str> {
        self.patterns.keys().map(|s| s.as_str()).collect()
    }

    /// Number of patterns in this spell book.
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    /// Compose multiple patterns into a single combined bytecode sequence.
    /// Resolves dependency order automatically.
    pub fn compose(&self, pattern_ids: &[&str]) -> Result<Vec<u8>, String> {
        let mut missing = Vec::new();
        let subset: HashMap<String, Pattern> = pattern_ids.iter()
            .filter_map(|&id| {
                if let Some(p) = self.patterns.get(id) {
                    Some((id.to_string(), p.clone()))
                } else {
                    missing.push(id.to_string());
                    None
                }
            })
            .collect();

        if !missing.is_empty() {
            return Err(format!("Missing patterns: {:?}", missing));
        }

        let order = crate::pattern::resolve_dependencies(&subset)
            .ok_or_else(|| "Circular dependency detected in composition".to_string())?;

        let mut composed = Vec::new();
        for id in &order {
            let pat = self.patterns.get(id).unwrap();
            composed.extend_from_slice(&pat.bytecode);
        }
        Ok(composed)
    }

    /// Export the spell book to a serializable map.
    pub fn export(&self) -> SpellBookExport {
        SpellBookExport {
            name: self.name.clone(),
            description: self.description.clone(),
            author: self.author.clone(),
            version: self.version.clone(),
            patterns: self.patterns.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        }
    }

    /// Import a spell book from exported data.
    pub fn from_export(data: &SpellBookExport) -> Self {
        let now = Self::now_ms();
        Self {
            name: data.name.clone(),
            description: data.description.clone(),
            author: data.author.clone(),
            version: data.version.clone(),
            patterns: data.patterns.clone(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Merge another spell book into this one. Returns number of new patterns imported.
    pub fn merge(&mut self, other: &SpellBook) -> usize {
        let mut count = 0;
        for (id, pattern) in &other.patterns {
            if !self.patterns.contains_key(id) {
                self.patterns.insert(id.clone(), pattern.clone());
                count += 1;
            }
        }
        if count > 0 {
            self.updated_at = Self::now_ms();
        }
        count
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// Serializable export format for spell books.
#[derive(Debug, Clone)]
pub struct SpellBookExport {
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub patterns: HashMap<String, Pattern>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_pattern(id: &str) -> Pattern {
        Pattern::new(id, id, &[id.as_bytes()[0], 0x01]).with_category("test")
    }

    #[test]
    fn spellbook_new() {
        let sb = SpellBook::new("Basics", "superz");
        assert_eq!(sb.name, "Basics");
        assert_eq!(sb.author, "superz");
        assert!(sb.is_empty());
    }

    #[test]
    fn spellbook_add_pattern() {
        let mut sb = SpellBook::new("Test", "author");
        let p = make_test_pattern("p1");
        assert!(sb.add_pattern(p));
        assert_eq!(sb.len(), 1);
    }

    #[test]
    fn spellbook_duplicate_add() {
        let mut sb = SpellBook::new("Test", "author");
        let p1 = make_test_pattern("p1");
        let p2 = make_test_pattern("p1");
        assert!(sb.add_pattern(p1));
        assert!(!sb.add_pattern(p2));
        assert_eq!(sb.len(), 1);
    }

    #[test]
    fn spellbook_remove_pattern() {
        let mut sb = SpellBook::new("Test", "author");
        sb.add_pattern(make_test_pattern("p1"));
        let removed = sb.remove_pattern("p1");
        assert!(removed.is_some());
        assert!(sb.is_empty());
        assert!(sb.remove_pattern("p1").is_none());
    }

    #[test]
    fn spellbook_get_pattern() {
        let mut sb = SpellBook::new("Test", "author");
        sb.add_pattern(make_test_pattern("p1"));
        assert!(sb.get_pattern("p1").is_some());
        assert!(sb.get_pattern("missing").is_none());
    }

    #[test]
    fn spellbook_pattern_ids() {
        let mut sb = SpellBook::new("Test", "author");
        sb.add_pattern(make_test_pattern("alpha"));
        sb.add_pattern(make_test_pattern("beta"));
        let ids = sb.pattern_ids();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn spellbook_compose_simple() {
        let mut sb = SpellBook::new("Test", "author");
        sb.add_pattern(Pattern::new("a", "A", &[0x01, 0x02]));
        sb.add_pattern(Pattern::new("b", "B", &[0x03, 0x04]));
        let result = sb.compose(&["a", "b"]);
        assert!(result.is_ok());
        let bc = result.unwrap();
        // Both patterns included, order depends on topo-sort (alphabetical when no deps)
        assert_eq!(bc.len(), 4);
        assert!(bc.contains(&0x01) && bc.contains(&0x02));
        assert!(bc.contains(&0x03) && bc.contains(&0x04));
    }

    #[test]
    fn spellbook_compose_missing() {
        let sb = SpellBook::new("Test", "author");
        let result = sb.compose(&["nonexistent"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing"));
    }

    #[test]
    fn spellbook_compose_with_deps() {
        let mut sb = SpellBook::new("Test", "author");
        sb.add_pattern(Pattern::new("a", "A", &[0x01]));
        sb.add_pattern(Pattern::new("b", "B", &[0x02]).with_dependencies(&["a"]));
        let result = sb.compose(&["b", "a"]);
        assert!(result.is_ok());
        // 'a' must come before 'b' due to dependency
        let bytecode = result.unwrap();
        assert_eq!(bytecode[0], 0x01); // a first
        assert_eq!(bytecode[1], 0x02); // b second
    }

    #[test]
    fn spellbook_export_import() {
        let mut sb = SpellBook::new("MyBook", "superz").with_description("A book of spells");
        sb.add_pattern(make_test_pattern("p1"));
        sb.add_pattern(make_test_pattern("p2"));

        let exported = sb.export();
        assert_eq!(exported.name, "MyBook");
        assert_eq!(exported.patterns.len(), 2);

        let imported = SpellBook::from_export(&exported);
        assert_eq!(imported.name, "MyBook");
        assert_eq!(imported.len(), 2);
        assert!(imported.get_pattern("p1").is_some());
    }

    #[test]
    fn spellbook_merge() {
        let mut sb1 = SpellBook::new("Book1", "a");
        sb1.add_pattern(make_test_pattern("shared"));
        sb1.add_pattern(make_test_pattern("only1"));

        let mut sb2 = SpellBook::new("Book2", "b");
        sb2.add_pattern(make_test_pattern("shared"));
        sb2.add_pattern(make_test_pattern("only2"));

        let count = sb1.merge(&sb2);
        assert_eq!(count, 1); // only "only2" is new
        assert_eq!(sb1.len(), 3);
    }

    #[test]
    fn spellbook_set_pattern_overwrites() {
        let mut sb = SpellBook::new("Test", "a");
        sb.add_pattern(Pattern::new("p1", "Old", &[0x01]));
        sb.set_pattern(Pattern::new("p1", "New", &[0x02]));
        assert_eq!(sb.get_pattern("p1").unwrap().name, "New");
        assert_eq!(sb.len(), 1);
    }

    #[test]
    fn spellbook_bump_pattern_version() {
        let mut sb = SpellBook::new("Test", "a");
        sb.add_pattern(Pattern::new("p1", "Test", &[0x01]));
        let pat = sb.get_pattern_mut("p1").unwrap();
        pat.bump_version("improvement");
        assert_eq!(pat.version, 2);
    }
}
