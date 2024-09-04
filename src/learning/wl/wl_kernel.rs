use crate::learning::{
    graphs::{CGraph, ColourDictionary},
    wl::{Neighbourhood, NeighbourhoodFactory, WlConfig, WlStatistics},
};
use ndarray::Array2;
use numpy::{PyArray2, PyArrayMethods};
use pyo3::{Bound, Python};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
enum Mode {
    Train,
    Evaluate,
}

/// A Weisfeiler-Lehman kernel.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WlKernel {
    /// The mode of the kernel. In training mode, the kernel will create new
    /// hashes for unseen subgraphs. In evaluation mode, the kernel will
    /// record statistics about the kernel but will not create new hashes.
    mode: Mode,
    /// The factory for creating neighbourhoods.
    neighbourhood_factory: NeighbourhoodFactory,
    /// Dimension of the Weisfeiler-Lehman algorithm.
    k: usize,
    /// The number of iterations to run the Weisfeiler-Lehman algorithm for.
    iters: usize,
    /// Mapping from subgraph hashes to colours.
    hashes: HashMap<Neighbourhood, i32>,
    /// The statistics of the kernel.
    #[serde(skip)]
    statistics: WlStatistics,
}

impl WlKernel {
    pub fn new(config: &WlConfig) -> Self {
        Self {
            mode: Mode::Train,
            neighbourhood_factory: NeighbourhoodFactory::new(config.set_or_multiset),
            k: 1, // UNIMPLEMENTED: Implement k-WL for k > 1.
            iters: config.iters,
            hashes: HashMap::new(),
            statistics: WlStatistics::new(),
        }
    }

    pub fn reset(&mut self) {
        self.hashes = HashMap::new();
        self.statistics = WlStatistics::new();
        self.mode = Mode::Train;
    }

    /// Compute colour histograms for some graph. This will run the
    /// Weisfeiler-Lehman algorithm on the graphs. The first time this is
    /// called, the kernel will be in training mode and will create new hashes
    /// for unseen subgraphs. Subsequent calls will be in evaluation mode and
    /// will record statistics about the kernel.
    pub fn compute_histograms(
        &mut self,
        graphs: &[CGraph],
        mut colour_dictionary: Option<&mut ColourDictionary>,
    ) -> Vec<HashMap<i32, usize>> {
        assert_eq!(self.k, 1, "k-WL not implemented yet for k > 1.");

        if self.mode == Mode::Train {
            self.hashes.clear();

            let mut old_dictionary = None;
            if let Some(colour_dictionary) = colour_dictionary.as_mut() {
                old_dictionary = Some(colour_dictionary.clone());
                colour_dictionary.clear();
            }

            let all_graph_colours = graphs
                .iter()
                .flat_map(|graph| graph.node_indices().map(|node| graph[node]))
                .collect::<HashSet<_>>();

            if all_graph_colours.is_empty() {
                self.mode = Mode::Evaluate;
                return vec![HashMap::new(); graphs.len()];
            }

            let max_graph_colour = graphs
                .iter()
                .map(|graph| graph.node_indices().map(|node| graph[node]).max().unwrap())
                .max();

            // Add the colours of the nodes to the hash map.
            let mut new_colour = 0;
            for graph_colour in 0..=max_graph_colour.unwrap() {
                if !all_graph_colours.contains(&graph_colour) {
                    continue;
                }
                self.hashes.insert(
                    self.neighbourhood_factory.create_neighbourhood(
                        graph_colour as i32,
                        vec![],
                        true,
                    ),
                    new_colour,
                );

                if let Some(colour_dictionary) = colour_dictionary.as_mut() {
                    colour_dictionary.insert(
                        new_colour,
                        old_dictionary
                            .as_ref()
                            .unwrap()
                            .get(graph_colour as i32)
                            .unwrap()
                            .clone(),
                    );
                }

                new_colour += 1;
            }
        }

        let mut histograms = vec![];
        let mut cur_colours = vec![];
        for graph in graphs {
            self.statistics.register_graph(graph.node_count() as i64);

            let mut histogram = HashMap::new();
            let mut cur_colour = HashMap::new();
            for node in graph.node_indices() {
                let colour_hash = self.get_hash_value(
                    self.neighbourhood_factory.create_neighbourhood(
                        graph[node] as i32,
                        vec![],
                        true,
                    ),
                    &mut colour_dictionary,
                );
                cur_colour.insert(node, colour_hash);
                histogram
                    .entry(colour_hash)
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
            }
            histograms.push(histogram);
            cur_colours.push(cur_colour);
        }

        for _ in 0..self.iters {
            for (graph_id, graph) in graphs.iter().enumerate() {
                let mut new_colours = HashMap::new();
                for node in graph.node_indices() {
                    let mut neighbour_colours = vec![];
                    for neighbour in graph.neighbors(node) {
                        let edge = graph.find_edge(node, neighbour).unwrap();
                        neighbour_colours.push((
                            cur_colours[graph_id][&neighbour],
                            *graph.edge_weight(edge).unwrap() as i32,
                        ));
                    }
                    let neighbourhood = self.neighbourhood_factory.create_neighbourhood(
                        cur_colours[graph_id][&node],
                        neighbour_colours,
                        false,
                    );
                    let colour_hash = self.get_hash_value(neighbourhood, &mut colour_dictionary);
                    new_colours.insert(node, colour_hash);
                    histograms[graph_id]
                        .entry(colour_hash)
                        .and_modify(|e| *e += 1)
                        .or_insert(1);
                }
                cur_colours[graph_id] = new_colours;
            }
        }

        if self.mode == Mode::Train {
            self.mode = Mode::Evaluate;
        }
        histograms
    }

    /// Convert the computed histograms to a feature matrix X as a 2D numpy
    /// array. The rows of the array correspond to the histograms of the graphs
    /// and the columns correspond to the counts in the histogram.
    pub fn convert_to_pyarray<'py>(
        &self,
        py: Python<'py>,
        histograms: &[HashMap<i32, f64>],
    ) -> Bound<'py, PyArray2<f64>> {
        let n = histograms.len();
        let d = self.hashes.len();
        let features = PyArray2::zeros_bound(py, [n, d], false);
        let mut features_readwrite = features.readwrite();
        for (i, histogram) in histograms.iter().enumerate() {
            for (&hash, &cnt) in histogram.iter() {
                if hash < 0 {
                    continue;
                }
                *features_readwrite.get_mut([i, hash as usize]).unwrap() = cnt;
            }
        }
        features
    }

    pub fn convert_to_ndarray(&self, histograms: &[HashMap<i32, f64>]) -> Array2<f64> {
        let n = histograms.len();
        let d = self.hashes.len();
        let mut features = Array2::zeros((n, d));
        for (i, histogram) in histograms.iter().enumerate() {
            for (&hash, &cnt) in histogram.iter() {
                // it's intentional that we let hashes be negative and ignores
                // the negative ones - they mean colours unseen from training
                // time
                if hash < 0 {
                    continue;
                }
                features[[i, hash as usize]] = cnt;
            }
        }
        features
    }

    fn get_hash_value(
        &mut self,
        neighbourhood: Neighbourhood,
        colour_dictionary: &mut Option<&mut ColourDictionary>,
    ) -> i32 {
        match self.mode {
            Mode::Train => match self.hashes.get(&neighbourhood) {
                Some(hash) => *hash,
                None => {
                    let new_hash = self.hashes.len() as i32;
                    if let Some(colour_dictionary) = colour_dictionary.as_mut() {
                        if !neighbourhood.from_base_graph {
                            let description = format!(
                                "{} with neighbours [{}]",
                                colour_dictionary.get(neighbourhood.node_colour).unwrap(),
                                neighbourhood
                                    .neighbour_colours
                                    .iter()
                                    .map(|(neighbour_colour, edge_colour)| format!(
                                        "({}, {})",
                                        colour_dictionary.get(*neighbour_colour).unwrap(),
                                        edge_colour
                                    ))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            );
                            colour_dictionary.insert(new_hash, description);
                        }
                    }

                    self.hashes.insert(neighbourhood, new_hash);

                    new_hash
                }
            },
            Mode::Evaluate => match self.hashes.get(&neighbourhood) {
                Some(hash) => {
                    self.statistics.increment_hit_colours();
                    *hash
                }
                None => {
                    self.statistics.increment_miss_colours();
                    // Return a bad value, this will cause all subsequent hashes
                    // to be bad as well, and hence paint an accurate picture of
                    // how many colours are missing.
                    -1
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::models::{PreprocessingOption, Preprocessor};
    use crate::learning::wl::SetOrMultiset;

    const SET_CONFIG: WlConfig = WlConfig {
        set_or_multiset: SetOrMultiset::Set,
        iters: 1,
    };

    const MULTISET_CONFIG: WlConfig = WlConfig {
        set_or_multiset: SetOrMultiset::Multiset,
        iters: 1,
    };

    #[test]
    fn starts_in_train_mode() {
        let kernel = WlKernel::new(&MULTISET_CONFIG);
        assert_eq!(kernel.mode, Mode::Train);
    }

    #[test]
    fn computing_histograms_changes_mode() {
        let mut kernel = WlKernel::new(&MULTISET_CONFIG);
        let graphs = vec![];
        kernel.compute_histograms(&graphs, None);
        assert_eq!(kernel.mode, Mode::Evaluate);

        // Second time should not change the mode.
        kernel.compute_histograms(&graphs, None);
        assert_eq!(kernel.mode, Mode::Evaluate);
    }

    #[test]
    fn computes_histograms_correctly_with_multiset() {
        let mut kernel = WlKernel::new(&MULTISET_CONFIG);
        let mut graph = CGraph::new_undirected();
        let node_0 = graph.add_node(0);
        let node_1 = graph.add_node(1);
        let node_2 = graph.add_node(0);
        graph.add_edge(node_0, node_1, 0);
        graph.add_edge(node_1, node_2, 0);

        let histograms = kernel.compute_histograms(&[graph.clone()], None);
        assert_eq!(histograms.len(), 1);
        assert_eq!(histograms[0].len(), 4);

        // Check that the histograms are the same when repeated.
        let repeated_histograms = kernel.compute_histograms(&[graph.clone()], None);
        assert_eq!(histograms, repeated_histograms);
    }

    #[test]
    fn computes_x_correctly_with_multiset() {
        let mut kernel = WlKernel::new(&MULTISET_CONFIG);
        // Need this to help convert to f64
        let mut preprocessor = Preprocessor::new(PreprocessingOption::None);

        let mut graph = CGraph::new_undirected();
        let node_0 = graph.add_node(0);
        let node_1 = graph.add_node(1);
        let node_2 = graph.add_node(0);
        graph.add_edge(node_0, node_1, 0);
        graph.add_edge(node_1, node_2, 0);

        let histograms = kernel.compute_histograms(&[graph.clone()], None);
        Python::with_gil(|py| {
            let x = kernel.convert_to_pyarray(py, &preprocessor.preprocess(histograms, true));
            assert_eq!(unsafe { x.as_slice().unwrap() }, &[2.0, 1.0, 2.0, 1.0]);
        });

        let mut graph2 = CGraph::new_undirected();
        let node_0 = graph2.add_node(0);
        let node_1 = graph2.add_node(1);
        let node_2 = graph2.add_node(0);
        let node_3 = graph2.add_node(0);
        graph2.add_edge(node_0, node_1, 0);
        graph2.add_edge(node_1, node_2, 0);
        graph2.add_edge(node_1, node_3, 0);

        let histograms2 = kernel.compute_histograms(&[graph2.clone()], None);
        Python::with_gil(|py| {
            let x = kernel.convert_to_pyarray(py, &preprocessor.preprocess(histograms2, true));
            assert_eq!(unsafe { x.as_slice().unwrap() }, &[3.0, 1.0, 3.0, 0.0]);
        });
    }

    #[test]
    fn computes_histograms_correctly_with_set() {
        let mut kernel = WlKernel::new(&SET_CONFIG);
        let mut graph = CGraph::new_undirected();
        let node_0 = graph.add_node(0);
        let node_1 = graph.add_node(1);
        let node_2 = graph.add_node(0);
        graph.add_edge(node_0, node_1, 0);
        graph.add_edge(node_1, node_2, 0);

        let histograms = kernel.compute_histograms(&[graph.clone()], None);
        assert_eq!(histograms.len(), 1);
        assert_eq!(histograms[0].len(), 4);

        // Check that the histograms are the same when repeated.
        let repeated_histograms = kernel.compute_histograms(&[graph.clone()], None);
        assert_eq!(histograms, repeated_histograms);
    }

    #[test]
    fn computes_x_correctly_with_set() {
        let mut kernel = WlKernel::new(&SET_CONFIG);
        // Need this to help convert to f64
        let mut preprocessor = Preprocessor::new(PreprocessingOption::None);

        let mut graph = CGraph::new_undirected();
        let node_0 = graph.add_node(0);
        let node_1 = graph.add_node(1);
        let node_2 = graph.add_node(0);
        graph.add_edge(node_0, node_1, 0);
        graph.add_edge(node_1, node_2, 0);

        let histograms = kernel.compute_histograms(&[graph.clone()], None);
        Python::with_gil(|py| {
            let x = kernel.convert_to_pyarray(py, &preprocessor.preprocess(histograms, true));
            assert_eq!(unsafe { x.as_slice().unwrap() }, &[2.0, 1.0, 2.0, 1.0]);
        });

        let mut graph2 = CGraph::new_undirected();
        let node_0 = graph2.add_node(0);
        let node_1 = graph2.add_node(1);
        let node_2 = graph2.add_node(0);
        let node_3 = graph2.add_node(0);
        graph2.add_edge(node_0, node_1, 0);
        graph2.add_edge(node_1, node_2, 0);
        graph2.add_edge(node_1, node_3, 0);

        let histograms2 = kernel.compute_histograms(&[graph2.clone()], None);
        Python::with_gil(|py| {
            let x = kernel.convert_to_pyarray(py, &preprocessor.preprocess(histograms2, true));
            assert_eq!(unsafe { x.as_slice().unwrap() }, &[3.0, 1.0, 3.0, 1.0]);
        });
    }
}
