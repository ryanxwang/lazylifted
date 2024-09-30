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
    ExtractGroundActionAndAddToPlan {
        plan: Rc<RefCell<HashSet<Action>>>,
        schema_index: usize,
    },
    AddGroundActionToPlan {
        plan: Rc<RefCell<HashSet<Action>>>,
        action: Action,
    },
}

impl PartialEq for Annotation {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Annotation::None, Annotation::None) => true,
            (
                Annotation::ExtractGroundActionAndAddToPlan {
                    plan: plan1,
                    schema_index: schema_index1,
                },
                Annotation::ExtractGroundActionAndAddToPlan {
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
            Annotation::ExtractGroundActionAndAddToPlan { plan, schema_index } => {
                let instantiation = program.extract_action_instantiation_from_fact(effect_fact_id);
                let action = Action::new(*schema_index, instantiation);
                plan.borrow_mut().insert(action);
            }
            Annotation::AddGroundActionToPlan { plan, action } => {
                plan.borrow_mut().insert(action.clone());
            }
        }
    }
}

impl Display for Annotation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Annotation::None => write!(f, "None"),
            Annotation::ExtractGroundActionAndAddToPlan {
                plan: _,
                schema_index,
            } => {
                write!(
                    f,
                    "ExtractGroundActionAndAddToPlan(schema_index: {})",
                    schema_index
                )
            }
            Annotation::AddGroundActionToPlan { plan: _, action } => {
                write!(f, "AddGroundActionToPlan(action: {:?})", action)
            }
        }
    }
}

pub type AnnotationGenerator = Box<dyn Fn(RuleCategory) -> Annotation>;
