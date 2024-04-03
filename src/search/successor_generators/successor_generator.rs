use crate::{
    search::successor_generators::{JoinSuccessorGenerator, NaiveJoinAlgorithm},
    Task,
};

/// A successor generator is responsible for generating the successors of a
/// given state.
pub enum SuccessorGenerators {
    Naive(JoinSuccessorGenerator<NaiveJoinAlgorithm>),
}

impl SuccessorGenerators {
    /// Create a new successor generator. Defaults to the naive implementation.
    pub fn new(task: &Task) -> Self {
        SuccessorGenerators::new_naive(task)
    }

    /// Create a successor generator from a name. This is useful for creating
    /// successor generators from command line arguments.
    pub fn from_name(name: &str) -> impl Fn(&Task) -> Self {
        match name {
            "naive" => SuccessorGenerators::new_naive,
            _ => panic!("Unknown successor generator: {}", name),
        }
    }

    /// Create a new naive successor generator. See
    /// [`super::NaiveJoinAlgorithm`] for more.
    pub fn new_naive(task: &Task) -> Self {
        SuccessorGenerators::Naive(JoinSuccessorGenerator::new(NaiveJoinAlgorithm::new(), task))
    }
}
