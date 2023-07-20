use num_rational::Rational64;

use crate::{intern::InternedString, list::List};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Symbol(InternedString),
    Lit(Lit),
    List(Vec<Self>),
    Lambda {
        params: Vec<Self>,
        body: Box<Self>,
    },
    Apply {
        func: Box<Self>,
        args: Vec<Self>,
    },
    Let {
        name: InternedString,
        value: Box<Self>,
        body: Box<Self>,
    },
    If {
        cond: Box<Self>,
        then: Box<Self>,
        else_: Box<Self>,
    },
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    Int(i64),
    Real(f64),
    Rational(Rational64),
    Bool(bool),
    Char(char),
    String(InternedString),
}
