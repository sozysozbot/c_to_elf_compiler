use std::collections::HashMap;

pub struct StrLitCollector {
    constant_pool_indices: HashMap<String, usize>,
    next_index: usize,
}

impl StrLitCollector {
    pub fn new() -> Self {
        StrLitCollector {
            constant_pool_indices: HashMap::new(),
            next_index: 0,
        }
    }

    pub fn insert_and_get_id(&mut self, s: String) -> usize {
        *self.constant_pool_indices.entry(s).or_insert_with(|| {
            let idx = self.next_index;
            self.next_index += 1;
            idx
        })
    }

    pub fn contains(&self, s: &str) -> bool {
        self.constant_pool_indices.contains_key(s)
    }
}

impl Default for StrLitCollector {
    fn default() -> Self {
        Self::new()
    }
}
