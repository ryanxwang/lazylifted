use std::fmt::Display;

use crate::search::datalog::{
    atom::Atom,
    rules::utils::{VariablePositionMap, VariableSource},
    Annotation,
};

/// A [`RuleCore`] represents the core of a Datalog rule. This should be used as
/// a component of a [`Rule`](crate::search::datalog::rules::Rule).
#[derive(Debug, Clone)]
pub struct RuleCore {
    /// The effect of the rule.
    effect: Atom,
    /// The conditions of the rule.
    conditions: Vec<Atom>,
    /// The weight of the rule.
    weight: f64,
    /// The annotation of the rule.
    annotation: Annotation,
    /// Whether the rule is ground, i.e. the effect contains no variables.
    is_effect_ground: bool,
    /// The mapping of variables to their positions in the effect atom.
    variable_position_map: VariablePositionMap,
    /// The lookup table for variables in the rule.
    variable_source: VariableSource,
}

impl RuleCore {
    /// Create a new [`RuleCore`] with the given effect, conditions, weight, and
    /// annotation.
    pub fn new(effect: Atom, conditions: Vec<Atom>, weight: f64, annotation: Annotation) -> Self {
        assert!(
            !conditions.is_empty(),
            "Datalog rules cannot have empty condition"
        );
        let is_effect_ground = effect.is_ground();
        let variable_position_map = VariablePositionMap::new(&effect);
        let variable_source = VariableSource::new(&effect, &conditions);
        Self {
            effect,
            conditions,
            weight,
            annotation,
            is_effect_ground,
            variable_position_map,
            variable_source,
        }
    }

    /// Get the effect of the rule.
    #[inline(always)]
    pub fn effect(&self) -> &Atom {
        &self.effect
    }

    /// Get the conditions of the rule.
    #[inline(always)]
    pub fn conditions(&self) -> &[Atom] {
        &self.conditions
    }

    /// Get whether the rule's effect (head) is ground.
    #[inline(always)]
    pub fn head_is_ground(&self) -> bool {
        self.is_effect_ground
    }

    #[inline(always)]
    pub fn weight(&self) -> f64 {
        self.weight
    }

    #[inline(always)]
    pub fn annotation(&self) -> &Annotation {
        &self.annotation
    }
}

impl PartialEq for RuleCore {
    fn eq(&self, other: &Self) -> bool {
        self.effect == other.effect
            && self.conditions == other.conditions
            && self.weight == other.weight
            && self.annotation == other.annotation
    }
}

impl Eq for RuleCore {}

impl Display for RuleCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <- ", self.effect)?;
        for (i, condition) in self.conditions.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", condition)?;
        }
        write!(
            f,
            "  | weight: {}; annotation: {}",
            self.weight, self.annotation
        )
    }
}
