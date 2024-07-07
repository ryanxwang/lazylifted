use crate::learning::wl::SetOrMultiset;
use serde::{Deserialize, Serialize};

/// A neighbourhood of a node in a graph, useful for just the Weisfeiler-Lehman
/// to hash down to a single value. Note that unlike the GOOSE implementation,
/// this does not include the edge colours to the neighbours.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Neighbourhood {
    node_colour: i32,
    neighbour_colours: Vec<(i32, i32)>,
}

/// A [`Neighbourhood`] factory that can create [`Neighbourhood`]s. This is used
/// to store configuration options for creating neighbourhoods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighbourhoodFactory {
    set_or_multiset: SetOrMultiset,
}

impl NeighbourhoodFactory {
    /// Create a new [`NeighbourhoodFactory`] with the given configuration.
    pub fn new(set_or_multiset: SetOrMultiset) -> Self {
        Self { set_or_multiset }
    }

    /// Create a new [`Neighbourhood`] from a node colour and a list of
    /// neighbour colours.
    pub fn create_neighbourhood(
        &self,
        node_colour: i32,
        mut neighbour_colours: Vec<(i32, i32)>,
    ) -> Neighbourhood {
        neighbour_colours.sort();

        match self.set_or_multiset {
            SetOrMultiset::Set => {
                neighbour_colours.dedup();
            }
            SetOrMultiset::Multiset => {}
        }

        Neighbourhood {
            node_colour,
            neighbour_colours,
        }
    }
}
