use std::collections::{HashMap, HashSet};

/// A reusable FLUX bytecode pattern with version tracking.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub id: String,
    pub name: String,
    pub bytecode: Vec<u8>,
    pub opcodes: Vec<String>,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub version: u32,
    pub dependencies: Vec<String>,
    pub changelog: Vec<String>,
    pub uses: u32,
    pub successes: u32,
    pub failures: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Pattern {
    pub fn new(id: &str, name: &str, bytecode: &[u8]) -> Self {
        let now = Self::now_ms();
        let opcodes = Self::extract_opcodes(bytecode);
        Self {
            id: id.to_string(),
            name: name.to_string(),
            bytecode: bytecode.to_vec(),
            opcodes,
            description: String::new(),
            category: "uncategorized".to_string(),
            tags: vec![],
            version: 1,
            dependencies: vec![],
            changelog: vec![],
            uses: 0,
            successes: 0,
            failures: 0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_category(mut self, cat: &str) -> Self {
        self.category = cat.to_string();
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn with_tags(mut self, tags: &[&str]) -> Self {
        self.tags = tags.iter().map(|t| t.to_string()).collect();
        self
    }

    pub fn with_dependencies(mut self, deps: &[&str]) -> Self {
        self.dependencies = deps.iter().map(|d| d.to_string()).collect();
        self
    }

    pub fn bump_version(&mut self, reason: &str) -> u32 {
        self.version += 1;
        self.updated_at = Self::now_ms();
        self.changelog.push(format!("v{}: {}", self.version, reason));
        self.version
    }

    pub fn success_rate(&self) -> f64 {
        if self.uses == 0 { return 1.0; }
        self.successes as f64 / self.uses as f64
    }

    pub fn record_use(&mut self, success: bool) {
        self.uses += 1;
        if success { self.successes += 1; } else { self.failures += 1; }
    }

    pub fn update_bytecode(&mut self, bytecode: &[u8]) {
        self.bytecode = bytecode.to_vec();
        self.opcodes = Self::extract_opcodes(bytecode);
        self.bump_version("bytecode updated");
    }

    fn extract_opcodes(bytecode: &[u8]) -> Vec<String> {
        if bytecode.is_empty() { return vec![]; }
        // Simulated opcode extraction: every 2 bytes is an opcode ID
        bytecode.chunks(2)
            .filter(|c| c.len() == 2)
            .map(|c| format!("OP_{:02X}{:02X}", c[0], c[1]))
            .collect()
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// Resolve dependency order for a set of patterns.
/// Returns pattern IDs in topological order. Returns None on circular dependency.
pub fn resolve_dependencies(patterns: &HashMap<String, Pattern>) -> Option<Vec<String>> {
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

    for id in patterns.keys() {
        in_degree.entry(id.as_str()).or_insert(0);
        adj.entry(id.as_str()).or_insert_with(Vec::new);
    }

    for (id, pat) in patterns {
        for dep in &pat.dependencies {
            if patterns.contains_key(dep) {
                adj.get_mut(dep.as_str()).unwrap().push(id.as_str());
                *in_degree.entry(id.as_str()).or_insert(0) += 1;
            }
        }
    }

    let mut queue: Vec<&str> = in_degree.iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();
    queue.sort(); // deterministic order

    let mut result = Vec::new();
    while let Some(id) = queue.pop() {
        result.push(id.to_string());
        for &next in adj.get(id).unwrap_or(&vec![]) {
            let deg = in_degree.get_mut(next).unwrap();
            *deg -= 1;
            if *deg == 0 {
                queue.push(next);
                queue.sort();
            }
        }
    }

    if result.len() != patterns.len() {
        None
    } else {
        Some(result)
    }
}

/// Detect circular dependencies among patterns.
pub fn detect_cycles(patterns: &HashMap<String, Pattern>) -> Vec<Vec<String>> {
    let mut cycles = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = HashSet::new();
    let mut path = Vec::new();

    for id in patterns.keys() {
        if !visited.contains(id) {
            dfs_cycle(id, patterns, &mut visited, &mut stack, &mut path, &mut cycles);
        }
    }
    cycles
}

fn dfs_cycle(
    id: &str,
    patterns: &HashMap<String, Pattern>,
    visited: &mut HashSet<String>,
    stack: &mut HashSet<String>,
    path: &mut Vec<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    visited.insert(id.to_string());
    stack.insert(id.to_string());
    path.push(id.to_string());

    if let Some(pat) = patterns.get(id) {
        for dep in &pat.dependencies {
            if !visited.contains(dep) {
                if patterns.contains_key(dep) {
                    dfs_cycle(dep, patterns, visited, stack, path, cycles);
                }
            } else if stack.contains(dep) {
                let idx = path.iter().position(|p| p == dep).unwrap_or(0);
                cycles.push(path[idx..].to_vec());
            }
        }
    }

    path.pop();
    stack.remove(id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_new() {
        let p = Pattern::new("p1", "Add", &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(p.id, "p1");
        assert_eq!(p.name, "Add");
        assert_eq!(p.version, 1);
        assert_eq!(p.uses, 0);
        assert_eq!(p.category, "uncategorized");
        assert_eq!(p.bytecode, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn pattern_extract_opcodes() {
        let p = Pattern::new("p1", "Test", &[0xAB, 0xCD, 0xEF, 0x01]);
        assert_eq!(p.opcodes.len(), 2);
        assert_eq!(p.opcodes[0], "OP_ABCD");
        assert_eq!(p.opcodes[1], "OP_EF01");
    }

    #[test]
    fn pattern_empty_bytecode() {
        let p = Pattern::new("p1", "Empty", &[]);
        assert!(p.opcodes.is_empty());
    }

    #[test]
    fn pattern_odd_bytecode() {
        let p = Pattern::new("p1", "Odd", &[0x01, 0x02, 0x03]);
        assert_eq!(p.opcodes.len(), 1); // only 1 complete 2-byte chunk
    }

    #[test]
    fn pattern_with_category() {
        let p = Pattern::new("p1", "Test", &[0x01, 0x02])
            .with_category("arithmetic");
        assert_eq!(p.category, "arithmetic");
    }

    #[test]
    fn pattern_with_tags() {
        let p = Pattern::new("p1", "Test", &[0x01, 0x02])
            .with_tags(&["math", "basic"]);
        assert_eq!(p.tags, vec!["math", "basic"]);
    }

    #[test]
    fn pattern_with_dependencies() {
        let p = Pattern::new("p1", "Test", &[0x01, 0x02])
            .with_dependencies(&["dep1", "dep2"]);
        assert_eq!(p.dependencies, vec!["dep1", "dep2"]);
    }

    #[test]
    fn pattern_bump_version() {
        let mut p = Pattern::new("p1", "Test", &[0x01, 0x02]);
        let v = p.bump_version("fixed bug");
        assert_eq!(v, 2);
        assert_eq!(p.version, 2);
        assert_eq!(p.changelog.len(), 1);
        assert!(p.changelog[0].contains("fixed bug"));
    }

    #[test]
    fn pattern_success_rate() {
        let mut p = Pattern::new("p1", "Test", &[0x01, 0x02]);
        assert_eq!(p.success_rate(), 1.0);
        p.record_use(true);
        p.record_use(true);
        p.record_use(false);
        assert!((p.success_rate() - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn pattern_update_bytecode() {
        let mut p = Pattern::new("p1", "Test", &[0x01, 0x02]);
        p.update_bytecode(&[0xFF, 0xEE, 0xDD, 0xCC]);
        assert_eq!(p.bytecode, vec![0xFF, 0xEE, 0xDD, 0xCC]);
        assert_eq!(p.version, 2);
        assert_eq!(p.opcodes.len(), 2);
    }

    #[test]
    fn resolve_dependencies_simple() {
        let mut pats = HashMap::new();
        pats.insert("a".to_string(), Pattern::new("a", "A", &[0x01, 0x02]));
        pats.insert("b".to_string(), Pattern::new("b", "B", &[0x03, 0x04]).with_dependencies(&["a"]));
        let order = resolve_dependencies(&pats).unwrap();
        assert!(order.iter().position(|x| x == "a").unwrap() < order.iter().position(|x| x == "b").unwrap());
    }

    #[test]
    fn resolve_dependencies_chain() {
        let mut pats = HashMap::new();
        pats.insert("a".to_string(), Pattern::new("a", "A", &[0x01]));
        pats.insert("b".to_string(), Pattern::new("b", "B", &[0x02]).with_dependencies(&["a"]));
        pats.insert("c".to_string(), Pattern::new("c", "C", &[0x03]).with_dependencies(&["b"]));
        let order = resolve_dependencies(&pats).unwrap();
        let pos_a = order.iter().position(|x| x == "a").unwrap();
        let pos_b = order.iter().position(|x| x == "b").unwrap();
        let pos_c = order.iter().position(|x| x == "c").unwrap();
        assert!(pos_a < pos_b && pos_b < pos_c);
    }

    #[test]
    fn resolve_dependencies_cycle() {
        let mut pats = HashMap::new();
        pats.insert("a".to_string(), Pattern::new("a", "A", &[0x01]).with_dependencies(&["b"]));
        pats.insert("b".to_string(), Pattern::new("b", "B", &[0x02]).with_dependencies(&["a"]));
        assert!(resolve_dependencies(&pats).is_none());
    }

    #[test]
    fn test_detect_cycles_found() {
        let mut pats = HashMap::new();
        pats.insert("a".to_string(), Pattern::new("a", "A", &[0x01]).with_dependencies(&["b"]));
        pats.insert("b".to_string(), Pattern::new("b", "B", &[0x02]).with_dependencies(&["a"]));
        let cycles = super::detect_cycles(&pats);
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_detect_no_cycles() {
        let mut pats = HashMap::new();
        pats.insert("a".to_string(), Pattern::new("a", "A", &[0x01]));
        pats.insert("b".to_string(), Pattern::new("b", "B", &[0x02]).with_dependencies(&["a"]));
        assert!(super::detect_cycles(&pats).is_empty());
    }

    #[test]
    fn resolve_missing_deps_ignored() {
        let mut pats = HashMap::new();
        pats.insert("a".to_string(), Pattern::new("a", "A", &[0x01]).with_dependencies(&["missing"]));
        let order = resolve_dependencies(&pats).unwrap();
        assert_eq!(order.len(), 1);
    }
}
