use numpy::PyArray1;
use pyo3::{
    types::{PyList, PyTuple},
    Bound, Python,
};

#[derive(Debug)]
pub struct RegressionTrainingData<F> {
    pub features: F,
    pub labels: Vec<f64>,
    pub noise: Option<Vec<f64>>,
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
    pub relation: RankingRelation,
}

#[derive(Debug)]
pub struct RankingTrainingData<F> {
    pub features: F,
    pub pairs: Vec<RankingPair>,
}

impl<F> RankingTrainingData<F> {
    pub fn with_features<G>(self, features: G) -> RankingTrainingData<G> {
        RankingTrainingData {
            features,
            pairs: self.pairs,
        }
    }

    pub fn pairs_for_python(&self) -> Bound<'static, PyList> {
        let py = unsafe { Python::assume_gil_acquired() };
        let py_tuples: Vec<Bound<PyTuple>> = self
            .pairs
            .iter()
            .map(|pair| {
                let relation = match pair.relation {
                    RankingRelation::Better => 1,
                    RankingRelation::BetterOrEqual => 0,
                };
                PyTuple::new_bound(py, [pair.i, pair.j, relation])
            })
            .collect();
        PyList::new_bound(py, py_tuples)
    }
}

#[derive(Debug)]
pub enum TrainingData<F> {
    Regression(RegressionTrainingData<F>),
    Ranking(RankingTrainingData<F>),
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
