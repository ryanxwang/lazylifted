//! Contains the [`Problem`] type.

use crate::parsed_types::{Name, NameLiteral, Objects, Requirements};

/// A domain-specific problem declaration.
///
/// ## Usages
/// This is the top-level type of a problem description within a [`Domain`](crate::Domain).
///
/// ## Example
/// ```
/// # use lazylifted::Parser;
/// # use lazylifted::parsed_types::*;
/// let input = r#"(define (problem get-paid)
///         (:domain briefcase-world)
///         (:init (place home) (place office)
///                (object p) (object d) (object b)
///                (at B home) (at P home) (at D home) (in P))
///         (:goal (and (at B office) (at D office) (at P home)))
///     )"#;
///
/// let problem = Problem::from_str(input).unwrap();
///
/// assert_eq!(problem.name(), "get-paid");
/// assert_eq!(problem.domain(), "briefcase-world");
/// assert!(problem.requirements().is_empty());
/// assert_eq!(problem.init().len(), 9);
/// assert_eq!(problem.goals().len(), 3);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    // The problem name.
    name: Name,
    /// The name of the [`Domain`] this problem belongs to.
    domain: Name,
    /// The optional list of requirements.
    requires: Requirements,
    /// The optional list of object declarations.
    objects: Objects,
    /// The initial state definition.
    init: Vec<NameLiteral>,
    /// The goal definition.
    goal: Vec<NameLiteral>,
}

impl Problem {
    /// Creates a new [`Problem`] instance.
    pub const fn new(
        name: Name,
        domain: Name,
        requires: Requirements,
        objects: Objects,
        init: Vec<NameLiteral>,
        goal: Vec<NameLiteral>,
    ) -> Self {
        Self {
            name,
            domain,
            requires,
            objects,
            init,
            goal,
        }
    }

    /// Creates a builder to easily construct problems.
    pub fn builder<P: Into<Name>, D: Into<Name>>(
        problem_name: P,
        domain_name: D,
        init: Vec<NameLiteral>,
        goal: Vec<NameLiteral>,
    ) -> Self {
        Self {
            name: problem_name.into(),
            domain: domain_name.into(),
            requires: Requirements::new([]), // TODO: Do we need to imply STRIPS?
            objects: Objects::default(),
            init,
            goal,
        }
    }

    /// Adds a list of requirements to the problem.
    pub fn with_requirements<R: Into<Requirements>>(mut self, requirements: R) -> Self {
        self.requires = requirements.into();
        self
    }

    /// Adds a list of object declarations to the problem.
    pub fn with_objects<O: Into<Objects>>(mut self, objects: O) -> Self {
        self.objects = objects.into();
        self
    }

    /// Returns the problem name.
    pub const fn name(&self) -> &Name {
        &self.name
    }

    /// Returns the domain name.
    pub const fn domain(&self) -> &Name {
        &self.domain
    }

    /// Returns the optional problem requirements.
    pub const fn requirements(&self) -> &Requirements {
        &self.requires
    }

    /// Returns the optional object declarations.
    pub const fn objects(&self) -> &Objects {
        &self.objects
    }

    /// Returns the initialization of the problem.
    pub const fn init(&self) -> &Vec<NameLiteral> {
        &self.init
    }

    /// Returns the goal statement of the problem.
    pub const fn goals(&self) -> &Vec<NameLiteral> {
        &self.goal
    }
}
