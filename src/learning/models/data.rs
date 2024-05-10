#[derive(Debug)]
pub struct RegressionTrainingData<F, T> {
    pub features: F,
    pub labels: T,
    pub noise: Option<Vec<f64>>,
}

impl<F, T> RegressionTrainingData<F, T> {
    pub fn with_features<G>(self, features: G) -> RegressionTrainingData<G, T> {
        RegressionTrainingData {
            features,
            labels: self.labels,
            noise: self.noise,
        }
    }

    pub fn with_labels<U>(self, labels: U) -> RegressionTrainingData<F, U> {
        RegressionTrainingData {
            features: self.features,
            labels,
            noise: self.noise,
        }
    }
}

#[derive(Debug)]
pub struct RankingTrainingData<F, T> {
    pub features: F,
    pub ranks: T,
    pub groups: Vec<usize>,
}

impl<F, T> RankingTrainingData<F, T> {
    pub fn with_features<G>(self, features: G) -> RankingTrainingData<G, T> {
        RankingTrainingData {
            features,
            ranks: self.ranks,
            groups: self.groups,
        }
    }

    pub fn with_ranks<U>(self, ranks: U) -> RankingTrainingData<F, U> {
        RankingTrainingData {
            features: self.features,
            ranks,
            groups: self.groups,
        }
    }
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
            TrainingData::Regression(data) => {
                TrainingData::Regression(data.with_features(features))
            }
            TrainingData::Ranking(data) => TrainingData::Ranking(data.with_features(features)),
        }
    }

    pub fn with_targets<U>(self, targets: U) -> TrainingData<F, U> {
        match self {
            TrainingData::Regression(data) => TrainingData::Regression(data.with_labels(targets)),
            TrainingData::Ranking(data) => TrainingData::Ranking(data.with_ranks(targets)),
        }
    }
}
