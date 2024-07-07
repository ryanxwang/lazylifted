use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SetOrMultiset {
    Set,
    Multiset,
}

impl Default for SetOrMultiset {
    fn default() -> Self {
        Self::Multiset
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct WlConfig {
    pub iters: usize,
    #[serde(default)]
    pub set_or_multiset: SetOrMultiset,
}

impl WlConfig {
    pub fn with_iters(self, iters: usize) -> Self {
        Self { iters, ..self }
    }
}
