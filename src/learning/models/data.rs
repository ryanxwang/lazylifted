use crate::learning::graphs::CGraph;
use numpy::{PyArray1, PyArray2, PyUntypedArrayMethods};
use pyo3::{
    types::{PyAnyMethods, PyList, PyNone, PyTuple},
    Bound, PyAny, Python,
};
use tracing::info;

#[derive(Debug)]
pub struct RegressionTrainingData<F> {
    pub features: F,
    pub labels: Vec<f64>,
    pub noise: Option<Vec<f64>>,
}

impl<'a> RegressionTrainingData<Bound<'a, PyArray2<f64>>> {
    pub fn log(&self) {
        info!(feature_shape = format!("{:?}", self.features.shape()));
        info!(labels_count = self.labels.len());
        info!(noise = self.noise.is_some());
    }
}

impl<F> RegressionTrainingData<F> {
    pub fn with_features<G>(self, features: G) -> RegressionTrainingData<G> {
        RegressionTrainingData {
            features,
            labels: self.labels,
            noise: self.noise,
        }
    }

    pub fn numpy_labels(&self) -> Bound<'static, PyArray1<f64>> {
        let py = unsafe { Python::assume_gil_acquired() };
        PyArray1::from_vec_bound(py, self.labels.clone())
    }
}

#[derive(Debug)]
pub enum RankingRelation {
    Better,
    BetterOrEqual,
}

#[derive(Debug)]
pub struct RankingPair {
    pub i: usize,
    pub j: usize,
    /// The relation between the two items. If `Better`, then the model should
    /// learn that `i` is better than `j`. If `BetterOrEqual`, then the model
    /// should learn that `i` is better or equal to `j`.
    pub relation: RankingRelation,
    /// A weight that is used to scale the importance of this pair. This can be
    /// used to give more importance to some pairs than others.
    pub importance: f64,
}

#[derive(Debug)]
pub struct RankingTrainingData<F> {
    pub features: F,
    pub pairs: Vec<RankingPair>,
    /// Optional group ids that identify which group each feature vector belongs
    /// to. If provided, the ranking model may be able to specialise within each
    /// group (i.e. treat them as different feature space and use different
    /// weights for each group).
    pub group_ids: Option<Vec<usize>>,
}

impl<'a> RankingTrainingData<Bound<'a, PyArray2<f64>>> {
    pub fn log(&self) {
        info!(feature_shape = format!("{:?}", self.features.shape()));
        info!(pairs_count = self.pairs.len());

        match &self.group_ids {
            Some(group_ids) => {
                let unique_groups = group_ids.iter().collect::<std::collections::HashSet<_>>();

                info!(unique_groups_count = unique_groups.len());
                info!(unique_groups = format!("{:?}", unique_groups));
            }
            None => {
                info!(unique_groups_count = "None");
            }
        }
    }
}

impl<F> RankingTrainingData<F> {
    pub fn with_features<G>(self, features: G) -> RankingTrainingData<G> {
        RankingTrainingData {
            features,
            pairs: self.pairs,
            group_ids: self.group_ids,
        }
    }

    pub fn pairs_for_python(&self) -> Bound<'static, PyList> {
        let py = unsafe { Python::assume_gil_acquired() };
        let py_tuples: Vec<Bound<PyAny>> = self
            .pairs
            .iter()
            .map(|pair| {
                let relation = match pair.relation {
                    RankingRelation::Better => 1,
                    RankingRelation::BetterOrEqual => 0,
                };
                let tuple = PyTuple::new_bound(py, [pair.i, pair.j, relation]);
                // Can't add importance when constructing since it is a
                // different type, which python allows but rust doesn't
                let tuple = tuple
                    .add(PyTuple::new_bound(py, [pair.importance]))
                    .unwrap();
                tuple
            })
            .collect();
        PyList::new_bound(py, py_tuples)
    }

    pub fn group_ids_for_python(&self) -> Bound<'static, PyAny> {
        let py = unsafe { Python::assume_gil_acquired() };
        match &self.group_ids {
            Some(group_ids) => {
                let py_list: Vec<usize> = group_ids.clone();
                PyList::new_bound(py, py_list).into_any()
            }
            None => PyNone::get_bound(py).to_owned().into_any(),
        }
    }
}

#[derive(Debug)]
pub enum TrainingData<F> {
    Regression(RegressionTrainingData<F>),
    Ranking(RankingTrainingData<F>),
}

impl<'a> TrainingData<Bound<'a, PyArray2<f64>>> {
    pub fn log(&self) {
        match self {
            TrainingData::Regression(data) => data.log(),
            TrainingData::Ranking(data) => data.log(),
        }
    }
}

impl<F> TrainingData<F> {
    pub fn features(&self) -> &F {
        match self {
            TrainingData::Regression(data) => &data.features,
            TrainingData::Ranking(data) => &data.features,
        }
    }

    pub fn with_features<G>(self, features: G) -> TrainingData<G> {
        match self {
            TrainingData::Regression(data) => {
                TrainingData::Regression(data.with_features(features))
            }
            TrainingData::Ranking(data) => TrainingData::Ranking(data.with_features(features)),
        }
    }
}

impl TrainingData<Vec<CGraph>> {
    pub fn mean_graph_size(&self) -> f64 {
        let total_size: usize = self.features().iter().map(|graph| graph.node_count()).sum();
        total_size as f64 / self.features().len() as f64
    }
}
