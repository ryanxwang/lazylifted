use std::{cell::RefCell, collections::HashSet, fmt::Display, rc::Rc};

use crate::search::{
    datalog::{fact::FactId, program::Program},
    Action,
};

/// The category of a rule, which tells the annotation generator what annotation
/// it should generate
#[derive(Debug, Clone, Copy)]
pub enum RuleCategory {
    ActionApplicability { schema_index: usize },
    ActionEffect,
    Goal,
}

#[derive(Debug, Clone)]
pub enum Annotation {
    None,
    AddToRelaxedPlan {
        plan: Rc<RefCell<HashSet<Action>>>,
        schema_index: usize,
    },
}

impl PartialEq for Annotation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Annotation::None, Annotation::None) => true,
            (
                Annotation::AddToRelaxedPlan {
                    plan: plan1,
                    schema_index: schema_index1,
                },
                Annotation::AddToRelaxedPlan {
                    plan: plan2,
                    schema_index: schema_index2,
                },
            ) => Rc::ptr_eq(plan1, plan2) && schema_index1 == schema_index2,
            _ => false,
        }
    }
}

impl Eq for Annotation {}

impl Annotation {
    pub fn execute(&self, effect_fact_id: FactId, program: &Program) {
        match self {
            Annotation::None => {}
            Annotation::AddToRelaxedPlan { plan, schema_index } => {
                let instantiation = program.extract_action_instantiation_from_fact(effect_fact_id);
                let action = Action::new(*schema_index, instantiation);
                plan.borrow_mut().insert(action);
            }
        }
    }
}

impl Display for Annotation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Annotation::None => write!(f, "None"),
            Annotation::AddToRelaxedPlan {
                plan: _,
                schema_index,
            } => {
                write!(f, "AddToRelaxedPlan(schema_index: {})", schema_index)
            }
        }
    }
}

pub type AnnotationGenerator = Box<dyn Fn(RuleCategory) -> Annotation>;
