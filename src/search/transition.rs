//! A transition is a change from a search node to another. Since search nodes
//! do not necessarily correspond to actions, a transition can be more general
//! than an action.

pub trait Transition {
    fn no_transition() -> Self;
}
