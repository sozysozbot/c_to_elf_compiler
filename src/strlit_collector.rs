use std::{collections::HashSet};

pub struct StrLitCollector(HashSet<String>);

impl StrLitCollector {
    pub fn new() -> Self {
        StrLitCollector(HashSet::new())
    }

    pub fn insert(&mut self, s: String) {
        self.0.insert(s);
    }

    pub fn contains(&self, s: &str) -> bool {
        self.0.contains(s)
    }

    pub fn take(&mut self) -> HashSet<String> {
        std::mem::take(&mut self.0)
    }
}

impl Default for StrLitCollector {
    fn default() -> Self {
        Self::new()
    }
}