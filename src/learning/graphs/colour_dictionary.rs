use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone)]
pub struct ColourDictionary {
    descriptions: HashMap<i32, String>,
}

impl Default for ColourDictionary {
    fn default() -> Self {
        Self::new()
    }
}

impl ColourDictionary {
    pub fn new() -> Self {
        Self {
            descriptions: HashMap::new(),
        }
    }

    pub fn insert(&mut self, colour: i32, descriptor: String) {
        self.descriptions.insert(colour, descriptor);
    }

    pub fn get(&self, colour: i32) -> Option<&String> {
        self.descriptions.get(&colour)
    }

    pub fn clear(&mut self) {
        self.descriptions.clear();
    }
}

impl Display for ColourDictionary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sorted_entries: Vec<_> = self.descriptions.iter().collect();
        sorted_entries.sort_by_key(|(k, _)| *k);

        writeln!(f, "[Colour] Description")?;
        for (colour, descriptor) in sorted_entries {
            writeln!(f, "[{:6}] {}", colour, descriptor)?;
        }

        Ok(())
    }
}
