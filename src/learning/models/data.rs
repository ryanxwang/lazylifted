#[derive(Debug)]
pub struct RegressionTrainingData<F, T> {
    pub features: F,
    pub labels: T,
    pub noise: Option<Vec<f64>>,
}

#[derive(Debug)]
pub struct RankingTrainingData<F, T> {
    pub features: F,
    pub ranks: T,
    pub groups: Vec<usize>,
}

#[derive(Debug)]
pub enum TrainingData<F, T> {
    Regression(RegressionTrainingData<F, T>),
    Ranking(RankingTrainingData<F, T>),
}

impl<F, T> TrainingData<F, T> {
    pub fn features(&self) -> &F {
        match self {
            TrainingData::Regression(data) => &data.features,
            TrainingData::Ranking(data) => &data.features,
        }
    }

    pub fn targets(&self) -> &T {
        match self {
            TrainingData::Regression(data) => &data.labels,
            TrainingData::Ranking(data) => &data.ranks,
        }
    }

    pub fn groups(&self) -> Option<&Vec<usize>> {
        match self {
            TrainingData::Regression(_) => None,
            TrainingData::Ranking(data) => Some(&data.groups),
        }
    }

    pub fn with_features<G>(self, features: G) -> TrainingData<G, T> {
        match self {
            TrainingData::Regression(data) => TrainingData::Regression(RegressionTrainingData {
                features,
                labels: data.labels,
                noise: data.noise,
            }),
            TrainingData::Ranking(data) => TrainingData::Ranking(RankingTrainingData {
                features,
                ranks: data.ranks,
                groups: data.groups,
            }),
        }
    }

    pub fn with_targets<U>(self, targets: U) -> TrainingData<F, U> {
        match self {
            TrainingData::Regression(data) => TrainingData::Regression(RegressionTrainingData {
                features: data.features,
                labels: targets,
                noise: data.noise,
            }),
            TrainingData::Ranking(data) => TrainingData::Ranking(RankingTrainingData {
                features: data.features,
                ranks: targets,
                groups: data.groups,
            }),
        }
    }
}
