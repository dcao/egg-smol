use std::num::NonZeroU32;

use crate::{ast::Symbol, Id};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
// FIXME this shouldn't be pub
pub struct Value {
    pub tag: Symbol,
    pub bits: u64,
}

impl Value {
    pub fn unit() -> Self {
        Value {
            tag: Symbol::new("Unit"),
            bits: 0,
        }
    }

    pub fn fake() -> Self {
        Value {
            tag: Symbol::new("__bogus__"),
            bits: 1234567890,
        }
    }

    pub fn from_id(tag: Symbol, id: Id) -> Self {
        Value {
            tag,
            bits: usize::from(id) as u64,
        }
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Self {
            tag: Symbol::from("i64"),
            bits: i as u64,
        }
    }
}

impl From<Symbol> for Value {
    fn from(s: Symbol) -> Self {
        Self {
            tag: Symbol::from("i64"),
            bits: NonZeroU32::from(s).get().into(),
        }
    }
}
