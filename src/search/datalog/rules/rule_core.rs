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
    pub fn effect(&self) -> &Atom {
        &self.effect
    }

    /// Get the conditions of the rule.
    pub fn conditions(&self) -> &[Atom] {
        &self.conditions
    }

    /// Get whether the rule's effect (head) is ground.
    pub fn head_is_ground(&self) -> bool {
        self.is_effect_ground
    }
}
