use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ColourDictionary {
    descriptor: HashMap<i32, String>,
}

impl Default for ColourDictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl ColourDictionary {
    pub fn new() -> Self {
        Self {
            descriptor: HashMap::new(),
        }
    }

    pub fn insert(&mut self, colour: i32, descriptor: String) {
        self.descriptor.insert(colour, descriptor);
    }

    pub fn get(&self, colour: i32) -> Option<&String> {
        self.descriptor.get(&colour)
    }
}
