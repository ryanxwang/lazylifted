use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use crate::search::datalog::{
    annotation,
    atom::Atom,
    fact::Fact,
    rules::{rule_core::RuleCore, RuleTrait},
    term::Term,
    Annotation,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JoinConditionPosition {
    First,
    Second,
}

impl JoinConditionPosition {
    pub fn other(&self) -> Self {
        match self {
            Self::First => Self::Second,
            Self::Second => Self::First,
        }
    }
}

impl TryFrom<usize> for JoinConditionPosition {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::First),
            1 => Ok(Self::Second),
            _ => Err(()),
        }
    }
}

impl From<JoinConditionPosition> for usize {
    fn from(value: JoinConditionPosition) -> Self {
        match value {
            JoinConditionPosition::First => 0,
            JoinConditionPosition::Second => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JoinRule {
    core: RuleCore,
    /// Maps a variable to its position in the two conditions, only contains
    /// variables that existing in both positions (the joining variables).
    joining_variable_positions: HashMap<Term, (usize, usize)>,
    /// Stores the facts that have been reached for each value of the joining
    /// variables. The first array entry corresponds to the first condition and
    /// the second array entry corresponds to the second condition.
    reached_facts_for_joining_variables: [HashMap<Vec<Term>, Vec<Fact>>; 2],
}

impl JoinRule {
    pub fn new(
        effect: Atom,
        conditions: (Atom, Atom),
        weight: f64,
        annotation: Annotation,
    ) -> Self {
        let core = RuleCore::new(effect, vec![conditions.0, conditions.1], weight, annotation);
        Self::new_from_core(core)
    }

    pub(super) fn new_from_core(core: RuleCore) -> Self {
        assert_eq!(core.conditions().len(), 2);
        let mut joining_variable_positions = HashMap::new();
        for (i, term1) in core.conditions()[0].arguments().iter().enumerate() {
            if term1.is_object() {
                continue;
            }
            for (j, term2) in core.conditions()[1].arguments().iter().enumerate() {
                if term1 == term2 {
                    joining_variable_positions.insert(term1.to_owned(), (i, j));
                }
            }
        }

        Self {
            core,
            joining_variable_positions,
            reached_facts_for_joining_variables: [HashMap::new(), HashMap::new()],
        }
    }

    pub fn joining_variable_positions(&self, position: JoinConditionPosition) -> HashSet<usize> {
        self.joining_variable_positions
            .values()
            .map(|(i, j)| match position {
                JoinConditionPosition::First => *i,
                JoinConditionPosition::Second => *j,
            })
            .collect()
    }

    pub fn register_reached_fact_for_joining_variables(
        &mut self,
        position: JoinConditionPosition,
        fact: Fact,
        joining_variable_values: Vec<Term>,
    ) {
        self.reached_facts_for_joining_variables[usize::from(position)]
            .entry(joining_variable_values)
            .or_default()
            .push(fact);
    }

    pub fn reached_facts_for_joining_variables(
        &self,
        position: JoinConditionPosition,
        joining_variable_values: &[Term],
    ) -> &[Fact] {
        self.reached_facts_for_joining_variables[usize::from(position)]
            .get(joining_variable_values)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn condition(&self, position: JoinConditionPosition) -> &Atom {
        &self.core.conditions()[usize::from(position)]
    }
}

impl Display for JoinRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({})", self.core)
    }
}

impl RuleTrait for JoinRule {
    fn core(&self) -> &RuleCore {
        &self.core
    }

    fn core_mut(&mut self) -> &mut RuleCore {
        &mut self.core
    }
}
