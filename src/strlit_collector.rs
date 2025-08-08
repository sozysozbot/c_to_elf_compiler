use std::collections::HashMap;

pub struct StrLitCollector {
    constant_pool_indices: HashMap<String, usize>,
    next_index: usize,
}

impl StrLitCollector {
    pub fn to_pool(self) -> Vec<String> {
        let mut pool: Vec<String> = vec![];        
        for (s, &i) in &self.constant_pool_indices {
            if i >= pool.len() {
                pool.resize(i + 1, String::new());
            }
            pool[i] = s.clone();
        }
        pool
    } 

    pub fn new() -> Self {
        Self {
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

    pub fn search_string_from_id(&self, id: usize) -> Option<&String> {
        self.constant_pool_indices.iter().find_map(|(s, &i)| if i == id { Some(s) } else { None })
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
