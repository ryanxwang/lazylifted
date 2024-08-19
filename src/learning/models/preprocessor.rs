use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum PreprocessingOption {
    /// No preprocessing
    None,
    /// For each feature, divide by std
    DivByStd,
    /// For each feature, subtract mean then divide by std
    StandardScaling,
}

impl Default for PreprocessingOption {
    fn default() -> Self {
        Self::None
    }
}

// This is ideally implemented as a trait or enum, but since it's not too
// important, we do the lazy thing of just having a struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Preprocessor {
    option: PreprocessingOption,
    std: Option<HashMap<i32, f64>>,
    mean: Option<HashMap<i32, f64>>,
}

impl Preprocessor {
    pub fn new(option: PreprocessingOption) -> Self {
        Self {
            option,
            std: None,
            mean: None,
        }
    }

    pub fn preprocess(
        &mut self,
        histograms: Vec<HashMap<i32, usize>>,
        is_training: bool,
    ) -> Vec<HashMap<i32, f64>> {
        match self.option {
            PreprocessingOption::None => histograms
                .into_iter()
                .map(|hist| hist.into_iter().map(|(k, v)| (k, v as f64)).collect())
                .collect(),
            PreprocessingOption::DivByStd => {
                if is_training {
                    self.compute_stats(&histograms);
                }

                let std = self.std.as_ref().unwrap();
                histograms
                    .into_iter()
                    .map(|hist| {
                        hist.into_iter()
                            .map(|(feature, count)| {
                                let std = std.get(&feature).unwrap();
                                (feature, (count as f64 / std))
                            })
                            .collect()
                    })
                    .collect()
            }
            PreprocessingOption::StandardScaling => {
                if is_training {
                    self.compute_stats(&histograms);
                }

                let mean = self.mean.as_ref().unwrap();
                let std = self.std.as_ref().unwrap();
                histograms
                    .into_iter()
                    .map(|hist| {
                        hist.into_iter()
                            .map(|(feature, count)| {
                                let mean = mean.get(&feature).unwrap();
                                let std = std.get(&feature).unwrap();
                                (feature, ((count as f64 - mean) / std))
                            })
                            .collect()
                    })
                    .collect()
            }
        }
    }

    fn compute_stats(&mut self, histograms: &[HashMap<i32, usize>]) {
        let mut mean = HashMap::new();

        let total = histograms.len() as f64;

        for hist in histograms {
            for (&feature, &count) in hist {
                mean.entry(feature)
                    .and_modify(|e| *e += count as f64)
                    .or_insert(count as f64);
            }
        }

        for (_, val) in mean.iter_mut() {
            *val /= total;
        }

        self.mean = Some(mean);

        let mut std = HashMap::new();
        for hist in histograms {
            for (&feature, &count) in hist {
                let mean = self.mean.as_ref().unwrap().get(&feature).unwrap();
                std.entry(feature)
                    .and_modify(|e| *e += (count as f64 - mean).powi(2))
                    .or_insert((count as f64 - mean).powi(2));
            }
        }

        for (_, val) in std.iter_mut() {
            *val = (*val / total).sqrt();

            const EPSILON: f64 = 1e-6;
            if -*val < EPSILON && *val < EPSILON {
                // since we are going to divide by std, if std is zero, we set
                // it to 1 such that the feature is not changed
                *val = 1.0;
            }
        }

        self.std = Some(std);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_float(hists: Vec<HashMap<i32, usize>>) -> Vec<HashMap<i32, f64>> {
        hists
            .into_iter()
            .map(|hist| hist.into_iter().map(|(k, v)| (k, v as f64)).collect())
            .collect()
    }

    #[test]
    fn none_preprocessing() {
        let mut preprocessor = Preprocessor::new(PreprocessingOption::None);
        let histograms = vec![
            [(1, 1), (2, 2)].iter().cloned().collect(),
            [(1, 3), (2, 4)].iter().cloned().collect(),
        ];

        let preprocessed = preprocessor.preprocess(histograms.clone(), true);

        assert_eq!(preprocessed, to_float(histograms));
    }

    #[test]
    fn div_by_std_preprocessing() {
        let mut preprocessor = Preprocessor::new(PreprocessingOption::DivByStd);
        let histograms = vec![
            [(1, 1), (2, 2)].iter().cloned().collect(),
            [(1, 3), (2, 6)].iter().cloned().collect(),
        ];

        let preprocessed = preprocessor.preprocess(histograms.clone(), true);

        // stds are [1.0, 2.0]
        let expected = vec![
            [(1, 1.0), (2, 1.0)].iter().cloned().collect(),
            [(1, 3.0), (2, 3.0)].iter().cloned().collect(),
        ];
        assert_eq!(preprocessed, expected);
    }

    #[test]
    fn standard_scaling_preprocessing() {
        let mut preprocessor = Preprocessor::new(PreprocessingOption::StandardScaling);
        let histograms = vec![
            [(1, 1), (2, 2)].iter().cloned().collect(),
            [(1, 3), (2, 6)].iter().cloned().collect(),
        ];

        let preprocessed = preprocessor.preprocess(histograms.clone(), true);

        // stds are [1.0, 2.0], means are [2.0, 4.0]
        let expected = vec![
            [(1, -1.0), (2, -1.0)].iter().cloned().collect(),
            [(1, 1.0), (2, 1.0)].iter().cloned().collect(),
        ];
        assert_eq!(preprocessed, expected);
    }
}
