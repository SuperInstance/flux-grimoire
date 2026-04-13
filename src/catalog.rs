use crate::pattern::Pattern;
use std::collections::HashMap;

/// An indexed library of common bytecode idioms with search capabilities.
#[derive(Debug, Clone)]
pub struct PatternCatalog {
    patterns: HashMap<String, Pattern>,
    /// category -> pattern IDs
    category_index: HashMap<String, Vec<String>>,
    /// opcode -> pattern IDs
    opcode_index: HashMap<String, Vec<String>>,
    /// tag -> pattern IDs
    tag_index: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub pattern_id: String,
    pub name: String,
    pub score: f64,
}

impl PatternCatalog {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            category_index: HashMap::new(),
            opcode_index: HashMap::new(),
            tag_index: HashMap::new(),
        }
    }

    /// Register a pattern in the catalog. Re-indexes if replacing.
    pub fn register(&mut self, pattern: Pattern) {
        let id = pattern.id.clone();
        // Remove old indices if replacing
        if let Some(old) = self.patterns.remove(&id) {
            Self::remove_from_index(&old, &mut self.category_index, &mut self.opcode_index, &mut self.tag_index);
        }
        // Build indices
        Self::add_to_index(&pattern, &mut self.category_index, &mut self.opcode_index, &mut self.tag_index);
        self.patterns.insert(id, pattern);
    }

    /// Remove a pattern from the catalog.
    pub fn unregister(&mut self, id: &str) -> Option<Pattern> {
        if let Some(pat) = self.patterns.remove(id) {
            Self::remove_from_index(&pat, &mut self.category_index, &mut self.opcode_index, &mut self.tag_index);
            Some(pat)
        } else {
            None
        }
    }

    /// Find a pattern by exact ID.
    pub fn get(&self, id: &str) -> Option<&Pattern> {
        self.patterns.get(id)
    }

    /// Get all patterns in a category.
    pub fn by_category(&self, category: &str) -> Vec<&Pattern> {
        self.category_index.get(category)
            .map(|ids| ids.iter().filter_map(|id| self.patterns.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all patterns that use a specific opcode.
    pub fn by_opcode(&self, opcode: &str) -> Vec<&Pattern> {
        self.opcode_index.get(opcode)
            .map(|ids| ids.iter().filter_map(|id| self.patterns.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all patterns with a specific tag.
    pub fn by_tag(&self, tag: &str) -> Vec<&Pattern> {
        self.tag_index.get(tag)
            .map(|ids| ids.iter().filter_map(|id| self.patterns.get(id)).collect())
            .unwrap_or_default()
    }

    /// Search patterns by name (substring match).
    pub fn search_by_name(&self, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        self.patterns.iter()
            .filter(|(_, p)| p.name.to_lowercase().contains(&query_lower))
            .map(|(_, p)| SearchResult {
                pattern_id: p.id.clone(),
                name: p.name.clone(),
                score: 1.0,
            })
            .collect()
    }

    /// Search patterns by multiple criteria: name, category, opcode, tag.
    /// Returns results sorted by relevance score.
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut results: Vec<(String, String, f64)> = Vec::new();

        for (id, p) in &self.patterns {
            let mut score = 0.0f64;

            // Name match
            if p.name.to_lowercase().contains(&query_lower) {
                score += 3.0;
            }

            // ID match
            if id.to_lowercase().contains(&query_lower) {
                score += 2.0;
            }

            // Category match
            if p.category.to_lowercase().contains(&query_lower) {
                score += 2.0;
            }

            // Opcode match
            for op in &p.opcodes {
                if op.to_lowercase().contains(&query_lower) {
                    score += 1.5;
                }
            }

            // Tag match
            for tag in &p.tags {
                if tag.to_lowercase().contains(&query_lower) {
                    score += 1.0;
                }
            }

            // Description match
            if p.description.to_lowercase().contains(&query_lower) {
                score += 0.5;
            }

            if score > 0.0 {
                results.push((id.clone(), p.name.clone(), score));
            }
        }

        results.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        results.into_iter()
            .map(|(id, name, score)| SearchResult { pattern_id: id, name, score })
            .collect()
    }

    /// List all unique categories.
    pub fn categories(&self) -> Vec<&str> {
        let mut cats: Vec<&str> = self.category_index.keys().map(|s| s.as_str()).collect();
        cats.sort();
        cats
    }

    /// List all unique opcodes across all patterns.
    pub fn opcodes(&self) -> Vec<&str> {
        let mut ops: Vec<&str> = self.opcode_index.keys().map(|s| s.as_str()).collect();
        ops.sort();
        ops
    }

    /// List all unique tags across all patterns.
    pub fn tags(&self) -> Vec<&str> {
        let mut tags: Vec<&str> = self.tag_index.keys().map(|s| s.as_str()).collect();
        tags.sort();
        tags
    }

    /// Total number of patterns in the catalog.
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    fn add_to_index(
        pat: &Pattern,
        cat_idx: &mut HashMap<String, Vec<String>>,
        op_idx: &mut HashMap<String, Vec<String>>,
        tag_idx: &mut HashMap<String, Vec<String>>,
    ) {
        cat_idx.entry(pat.category.clone()).or_default().push(pat.id.clone());
        for op in &pat.opcodes {
            op_idx.entry(op.clone()).or_default().push(pat.id.clone());
        }
        for tag in &pat.tags {
            tag_idx.entry(tag.clone()).or_default().push(pat.id.clone());
        }
    }

    fn remove_from_index(
        pat: &Pattern,
        cat_idx: &mut HashMap<String, Vec<String>>,
        op_idx: &mut HashMap<String, Vec<String>>,
        tag_idx: &mut HashMap<String, Vec<String>>,
    ) {
        if let Some(ids) = cat_idx.get_mut(&pat.category) {
            ids.retain(|id| id != &pat.id);
            if ids.is_empty() { cat_idx.remove(&pat.category); }
        }
        for op in &pat.opcodes {
            if let Some(ids) = op_idx.get_mut(op) {
                ids.retain(|id| id != &pat.id);
                if ids.is_empty() { op_idx.remove(op); }
            }
        }
        for tag in &pat.tags {
            if let Some(ids) = tag_idx.get_mut(tag) {
                ids.retain(|id| id != &pat.id);
                if ids.is_empty() { tag_idx.remove(tag); }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pattern(id: &str, name: &str, cat: &str, tags: &[&str], bytecode: &[u8]) -> Pattern {
        Pattern::new(id, name, bytecode)
            .with_category(cat)
            .with_tags(tags)
    }

    #[test]
    fn catalog_new_empty() {
        let cat = PatternCatalog::new();
        assert!(cat.is_empty());
    }

    #[test]
    fn catalog_register_and_get() {
        let mut cat = PatternCatalog::new();
        let p = make_pattern("p1", "Add Pattern", "arithmetic", &["math"], &[0x01, 0x02]);
        cat.register(p);
        assert_eq!(cat.len(), 1);
        assert!(cat.get("p1").is_some());
        assert!(cat.get("missing").is_none());
    }

    #[test]
    fn catalog_unregister() {
        let mut cat = PatternCatalog::new();
        cat.register(make_pattern("p1", "Test", "cat1", &[], &[0x01]));
        let removed = cat.unregister("p1");
        assert!(removed.is_some());
        assert!(cat.is_empty());
        assert!(cat.unregister("p1").is_none());
    }

    #[test]
    fn catalog_by_category() {
        let mut cat = PatternCatalog::new();
        cat.register(make_pattern("a1", "Add", "arithmetic", &[], &[0x01]));
        cat.register(make_pattern("a2", "Sub", "arithmetic", &[], &[0x02]));
        cat.register(make_pattern("b1", "Print", "io", &[], &[0x03]));
        assert_eq!(cat.by_category("arithmetic").len(), 2);
        assert_eq!(cat.by_category("io").len(), 1);
        assert_eq!(cat.by_category("nonexistent").len(), 0);
    }

    #[test]
    fn catalog_by_opcode() {
        let mut cat = PatternCatalog::new();
        // Both patterns use opcode OP_0102 from bytecode [0x01, 0x02]
        cat.register(make_pattern("p1", "First", "cat", &[], &[0x01, 0x02]));
        cat.register(make_pattern("p2", "Second", "cat", &[], &[0x01, 0x02, 0x03, 0x04]));
        assert_eq!(cat.by_opcode("OP_0102").len(), 2);
        assert_eq!(cat.by_opcode("OP_0304").len(), 1);
    }

    #[test]
    fn catalog_by_tag() {
        let mut cat = PatternCatalog::new();
        cat.register(make_pattern("p1", "Fast", "perf", &["fast", "basic"], &[0x01]));
        cat.register(make_pattern("p2", "Slow", "perf", &["slow"], &[0x02]));
        assert_eq!(cat.by_tag("fast").len(), 1);
        assert_eq!(cat.by_tag("slow").len(), 1);
        assert_eq!(cat.by_tag("basic").len(), 1);
    }

    #[test]
    fn catalog_search_by_name() {
        let mut cat = PatternCatalog::new();
        cat.register(make_pattern("p1", "Addition", "math", &[], &[0x01]));
        cat.register(make_pattern("p2", "Subtraction", "math", &[], &[0x02]));
        cat.register(make_pattern("p3", "Print Line", "io", &[], &[0x03]));
        let results = cat.search_by_name("add");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].pattern_id, "p1");
    }

    #[test]
    fn catalog_search_multi_criteria() {
        let mut cat = PatternCatalog::new();
        cat.register(make_pattern("p1", "FastAdd", "math", &["fast"], &[0x01, 0x02]));
        let results = cat.search("math");
        // Should match on category "math" (score 2.0)
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn catalog_search_sorted_by_relevance() {
        let mut cat = PatternCatalog::new();
        cat.register(make_pattern("p1", "math_op", "misc", &["math"], &[0x01]));
        cat.register(make_pattern("p2", "math_add", "math", &[], &[0x02]));
        // p2 matches name("math") + category("math") = 5.0
        // p1 matches name("math") + tag("math") = 4.0
        let results = cat.search("math");
        assert_eq!(results.len(), 2);
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn catalog_search_no_match() {
        let mut cat = PatternCatalog::new();
        cat.register(make_pattern("p1", "Test", "cat", &[], &[0x01]));
        assert!(cat.search("xyznonexistent").is_empty());
    }

    #[test]
    fn catalog_categories() {
        let mut cat = PatternCatalog::new();
        cat.register(make_pattern("p1", "A", "arithmetic", &[], &[0x01]));
        cat.register(make_pattern("p2", "B", "io", &[], &[0x02]));
        let cats = cat.categories();
        assert_eq!(cats.len(), 2);
    }

    #[test]
    fn catalog_opcodes() {
        let mut cat = PatternCatalog::new();
        cat.register(make_pattern("p1", "A", "cat", &[], &[0xAB, 0xCD]));
        let ops = cat.opcodes();
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0], "OP_ABCD");
    }

    #[test]
    fn catalog_reregister_updates_indices() {
        let mut cat = PatternCatalog::new();
        cat.register(make_pattern("p1", "Old", "cat1", &["tag1"], &[0x01, 0x02]));
        // Re-register with different category/tag
        cat.register(make_pattern("p1", "New", "cat2", &["tag2"], &[0x03, 0x04]));
        assert_eq!(cat.len(), 1);
        assert_eq!(cat.by_category("cat1").len(), 0);
        assert_eq!(cat.by_category("cat2").len(), 1);
        assert_eq!(cat.by_tag("tag1").len(), 0);
        assert_eq!(cat.by_tag("tag2").len(), 1);
    }

    #[test]
    fn catalog_unregister_cleans_indices() {
        let mut cat = PatternCatalog::new();
        cat.register(make_pattern("p1", "Only", "unique_cat", &["unique_tag"], &[0xFF, 0xEE]));
        cat.unregister("p1");
        assert!(cat.is_empty());
        assert!(cat.by_category("unique_cat").is_empty());
        assert!(cat.by_tag("unique_tag").is_empty());
        assert!(cat.opcodes().is_empty());
    }

    #[test]
    fn catalog_tags_list() {
        let mut cat = PatternCatalog::new();
        cat.register(make_pattern("p1", "A", "cat", &["alpha", "beta"], &[0x01]));
        cat.register(make_pattern("p2", "B", "cat", &["beta", "gamma"], &[0x02]));
        let tags = cat.tags();
        assert_eq!(tags.len(), 3);
    }
}
