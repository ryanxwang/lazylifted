//! Contains the [`Domain`] type.

use crate::parsed_types::{ActionDefinition, Constants, PredicateDefinition, Requirements};
use crate::parsed_types::{Name, Types};

/// The `Domain` type specifies a problem domain in which to plan.
///
/// ## Usage
/// This is the top-level type of a domain description. See also [`Problem`](crate::Problem).
///
/// ## Example
/// ```
/// # use lazylifted::{Domain, Name, Parser};
/// let input = r#"
/// (define
///     (domain sokoban)
///     (:requirements :typing)
///     (:types location direction box)
///
///     (:constants down up left right - direction)
///
///     (:predicates
///          (at-robot ?l - location)
///          (at ?o - box ?l - location)
///          (adjacent ?l1 - location ?l2 - location ?d - direction)
///          (clear ?l - location)
///     )
///
///     (:action move
///         :parameters (?from - location ?to - location ?dir - direction)
///         :precondition (and (clear ?to) (at-robot ?from) (adjacent ?from ?to ?dir))
///         :effect (and (at-robot ?to) (not (at-robot ?from)))
///     )
///             
///     (:action push
///         :parameters  (?rloc - location ?bloc - location ?floc - location ?dir - direction ?b - box)
///         :precondition (and (at-robot ?rloc) (at ?b ?bloc) (clear ?floc)
///                       (adjacent ?rloc ?bloc ?dir) (adjacent ?bloc ?floc ?dir))
///
///         :effect (and (at-robot ?bloc) (at ?b ?floc) (clear ?bloc)
///                 (not (at-robot ?rloc)) (not (at ?b ?bloc)) (not (clear ?floc)))
///     )
///)"#;
///
/// let domain = Domain::from_str(input).unwrap();
///
/// assert_eq!(domain.name(), &Name::new("sokoban"));
/// assert_eq!(domain.requirements().len(), 1);
/// assert_eq!(domain.types().len(), 3);
/// assert_eq!(domain.constants().len(), 4);
/// assert_eq!(domain.predicates().len(), 4);
/// assert_eq!(domain.actions().len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Domain {
    /// The domain name.
    name: Name,
    /// The specified requirements.
    requirements: Requirements,
    /// The optional type declarations.
    ///
    /// ## Requirements
    /// Requires [Typing](crate::Requirement::Typing).
    types: Types,
    /// The optional constant declarations.
    constants: Constants,
    /// The predicate definitions.
    predicates: Vec<PredicateDefinition>,
    /// The action definitions.
    actions: Vec<ActionDefinition>,
}

impl Domain {
    /// Creates a builder to easily construct [`Domain`] instances.
    pub fn builder<T: Into<Vec<ActionDefinition>>>(name: Name, actions: T) -> Self {
        Self {
            name,
            requirements: Requirements::default(),
            types: Types::default(),
            constants: Constants::default(),
            predicates: Vec::default(),
            actions: actions.into(),
        }
    }

    /// Adds a list of optional domain requirements.
    pub fn with_requirements(mut self, requirements: Requirements) -> Self {
        self.requirements = requirements;
        self
    }

    /// Adds a list of optional type declarations.
    pub fn with_types<T: Into<Types>>(mut self, types: T) -> Self {
        self.types = types.into();
        self
    }

    /// Adds a list of optional constant declarations.
    pub fn with_constants<C: Into<Constants>>(mut self, constants: C) -> Self {
        self.constants = constants.into();
        self
    }

    /// Adds a list of optional predicate definitions.
    pub fn with_predicates<P: Into<Vec<PredicateDefinition>>>(mut self, predicates: P) -> Self {
        self.predicates = predicates.into();
        self
    }

    /// Gets the domain name.
    pub const fn name(&self) -> &Name {
        &self.name
    }

    /// Returns the optional domain requirements.
    /// If no requirements were specified by the domain, [STRIPS](crate::Requirement::Strips) is implied.
    pub const fn requirements(&self) -> &Requirements {
        &self.requirements
    }

    /// Returns the optional type declarations.
    /// ## Requirements
    /// Requires [Typing](crate::Requirement::Typing).
    pub const fn types(&self) -> &Types {
        &self.types
    }

    /// Returns the optional constant definitions.
    pub const fn constants(&self) -> &Constants {
        &self.constants
    }

    /// Returns the optional predicate definitions.
    pub const fn predicates(&self) -> &Vec<PredicateDefinition> {
        &self.predicates
    }

    /// Returns the domain structure definitions.
    pub const fn actions(&self) -> &Vec<ActionDefinition> {
        &self.actions
    }
}

impl AsRef<Requirements> for Domain {
    fn as_ref(&self) -> &Requirements {
        &self.requirements
    }
}

impl AsRef<Types> for Domain {
    fn as_ref(&self) -> &Types {
        &self.types
    }
}

impl AsRef<Vec<PredicateDefinition>> for Domain {
    fn as_ref(&self) -> &Vec<PredicateDefinition> {
        &self.predicates
    }
}

impl AsRef<Vec<ActionDefinition>> for Domain {
    fn as_ref(&self) -> &Vec<ActionDefinition> {
        &self.actions
    }
}
