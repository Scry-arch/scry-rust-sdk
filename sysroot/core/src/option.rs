//! `Option`.

use crate::clone::Clone;
use crate::cmp::PartialEq;
use crate::marker::Copy;
use crate::panicking;

pub enum Option<T> {
    Some(T),
    None,
}

pub use self::Option::{None, Some};

impl<T> Option<T> {
    pub fn is_some(&self) -> bool {
        match self {
            Some(_) => true,
            None => false,
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Some(_) => false,
            None => true,
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            Some(t) => t,
            None => panicking::panic("called `Option::unwrap()` on a `None` value"),
        }
    }
}

impl<T: Copy> Copy for Option<T> {}

impl<T: Clone> Clone for Option<T> {
    fn clone(&self) -> Self {
        match self {
            Some(t) => Some(t.clone()),
            None => None,
        }
    }
}

impl<T: PartialEq> PartialEq for Option<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Some(lhs), Some(rhs)) => *lhs == *rhs,
            (None, None) => true,
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        match (self, other) {
            (Some(lhs), Some(rhs)) => *lhs != *rhs,
            (None, None) => false,
            _ => true,
        }
    }
}
