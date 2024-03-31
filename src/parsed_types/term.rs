use crate::parsed_types::Name;
use crate::parsed_types::Variable;

/// A term, i.e. a [`Name`], [`Variable`].
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Term {
    Name(Name),
    Variable(Variable),
}

impl Term {
    pub const fn new_name(name: Name) -> Self {
        Self::Name(name)
    }

    pub const fn new_variable(var: Variable) -> Self {
        Self::Variable(var)
    }
}

impl From<Name> for Term {
    fn from(value: Name) -> Self {
        Self::Name(value)
    }
}

impl From<Variable> for Term {
    fn from(value: Variable) -> Self {
        Self::Variable(value)
    }
}
