use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Rem, Sub},
    rc::Rc,
};

use num_bigint::BigInt;
use num_rational::Rational64;

use crate::interpreter::runtime_error::{Result, RuntimeError};

use self::env::Env;

pub mod env;

// A Lisp S-expression
#[repr(C)]
#[derive(Clone)]
pub enum Sexpr {
    // A Lisp atom
    Atom(Atom),

    // A Lisp list represented as a singly-linked list of conses
    List(List),

    // A Lisp lambda only constructed in eval
    Lambda {
        env: Rc<RefCell<Env>>,
        args: Vec<Self>,
        body: Box<Self>,
    },

    // A native Rust function only constructed in env
    NativeFn(fn(env: Rc<RefCell<Env>>, args: Vec<Sexpr>) -> Result<Sexpr>),
}

impl Sexpr {
    pub fn is_special_form(&self) -> bool {
        if let Sexpr::Atom(Atom::Sym(s)) = &self {
            return match s.as_str() {
                "def" | "let" | "fn" | "quote" | "if" => true,
                _ => false,
            };
        }
        false
    }

    pub fn get_special_form(&self) -> Option<String> {
        if let Sexpr::Atom(Atom::Sym(s)) = &self {
            return match s.as_str() {
                "def" | "let" | "fn" | "quote" | "if" => Some(s.clone()),
                _ => None,
            };
        }
        None
    }
}

impl Display for Sexpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Atom(a) => write!(f, "{}", a),
            Self::List(head) => write!(f, "{}", head),
            Self::Lambda { env, args, body } => write!(f, "<#fn>"),
            Self::NativeFn(_) => write!(f, "NativeFn"),
        }
    }
}

impl Debug for Sexpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Atom(a) => write!(f, "{}", a),
            Self::List(l) => write!(f, "{:?}", l),
            Self::Lambda { env, args, body } => todo!(),
            Self::NativeFn(nf) => write!(f, "{:?}", nf),
        }
    }
}

impl PartialEq for Sexpr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Atom(a1), Self::Atom(a2)) => *a1 == *a2,
            _ => false,
        }
    }
}

impl Eq for Sexpr {}

pub const NIL: Sexpr = Sexpr::List(List { head: None });

#[derive(Clone, Default, PartialEq)]
pub struct List {
    head: Option<Box<Cons>>,
}

impl List {
    pub fn new(head: Option<Box<Cons>>) -> Self {
        Self { head }
    }
}

impl Display for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.head {
            Some(h) => write!(f, "({})", h),
            None => write!(f, "Nil"),
        }
    }
}

impl Debug for List {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.head {
            Some(h) => write!(f, "{:?}", h),
            None => write!(f, "Nil"),
        }
    }
}

impl IntoIterator for List {
    type Item = Sexpr;

    type IntoIter = ConsIter;

    fn into_iter(self) -> Self::IntoIter {
        ConsIter(self.head.clone())
    }
}

#[derive(Clone, PartialEq)]
pub struct Cons {
    pub car: Sexpr,
    pub cdr: Option<Box<Cons>>,
}

impl Display for Cons {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.cdr {
            Some(cdr) => write!(f, "{} {}", self.car, cdr.as_ref()),
            None => write!(f, "{}", self.car),
        }
    }
}

impl Debug for Cons {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.cdr {
            Some(cdr) => write!(f, "({} . {:?})", self.car, cdr.as_ref()),
            None => write!(f, "({:?} . Nil)", self.car),
        }
    }
}

#[derive(Clone)]
pub struct ConsIter(Option<Box<Cons>>);

impl Iterator for ConsIter {
    type Item = Sexpr;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.clone().map(|cons| {
            let sexpr = cons.car.clone();

            self.0 = cons.cdr.clone();

            sexpr
        })
    }
}

impl ExactSizeIterator for ConsIter {
    fn len(&self) -> usize {
        let mut length: usize = 0;

        self.clone().for_each(|_| length += 1);

        length
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    Sym(String),
    Lit(Lit),
}

impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Atom::Sym(s) => write!(f, "{}", s),
            Atom::Lit(l) => write!(f, "{}", l),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    Number(Number),
    Bool(bool),
    Str(String),
}

impl Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lit::Number(n) => write!(f, "{}", n),
            Lit::Bool(b) => write!(f, "{}", b),
            Lit::Str(s) => write!(f, "\"{}\"", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Number {
    Fixnum(i64),
    Float(f64),
    Rational(Rational64),
    Bignum(BigInt),
}

impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Fixnum(n) => write!(f, "{}", n),
            Number::Float(d) => write!(f, "{}", d),
            Number::Rational(r) => write!(f, "{}", r),
            Number::Bignum(b) => write!(f, "{}", b),
        }
    }
}

impl Add for Number {
    type Output = Result<Number>;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Fixnum(l), Number::Fixnum(r)) => {
                if let Some(sum) = l.checked_add(r) {
                    Ok(Number::Fixnum(sum))
                } else {
                    Ok(Number::Bignum(BigInt::from(l) + r))
                }
            }
            (Number::Fixnum(l), Number::Float(r)) => Ok(Number::Float(l as f64 + r)),
            (Number::Fixnum(l), Number::Rational(r)) => {
                Ok(Number::Rational(Rational64::from(l) + r))
            }
            (Number::Fixnum(l), Number::Bignum(r)) => Ok(Number::Bignum(l + r)),
            (Number::Float(l), Number::Fixnum(r)) => Ok(Number::Float(l + r as f64)),
            (Number::Float(l), Number::Float(r)) => Ok(Number::Float(l + r)),
            (Number::Float(l), Number::Rational(r)) => {
                Ok(Number::Float(l + *r.numer() as f64 / *r.denom() as f64))
            }
            (Number::Rational(l), Number::Fixnum(r)) => {
                Ok(Number::Rational(l + Rational64::from(r)))
            }
            (Number::Rational(l), Number::Float(r)) => {
                Ok(Number::Float(*l.numer() as f64 / *l.denom() as f64 + r))
            }
            (Number::Rational(l), Number::Rational(r)) => Ok(Number::Rational(l + r)),
            (Number::Bignum(l), Number::Fixnum(r)) => Ok(Number::Bignum(l + r)),
            (Number::Bignum(l), Number::Bignum(r)) => Ok(Number::Bignum(l + r)),
            (Number::Bignum(_), Number::Float(_))
            | (Number::Float(_), Number::Bignum(_))
            | (Number::Bignum(_), Number::Rational(_))
            | (Number::Rational(_), Number::Bignum(_)) => Err(RuntimeError::NumberOverflowError),
        }
    }
}

impl Sub for Number {
    type Output = Result<Number>;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Fixnum(l), Number::Fixnum(r)) => {
                if let Some(diff) = l.checked_sub(r) {
                    Ok(Number::Fixnum(diff))
                } else {
                    Ok(Number::Bignum(BigInt::from(l) - r))
                }
            }
            (Number::Fixnum(l), Number::Float(r)) => Ok(Number::Float(l as f64 - r)),
            (Number::Fixnum(l), Number::Rational(r)) => {
                Ok(Number::Rational(Rational64::from(l) - r))
            }
            (Number::Fixnum(l), Number::Bignum(r)) => Ok(Number::Bignum(l - r)),
            (Number::Float(l), Number::Fixnum(r)) => Ok(Number::Float(l - r as f64)),
            (Number::Float(l), Number::Float(r)) => Ok(Number::Float(l - r)),
            (Number::Float(l), Number::Rational(r)) => {
                Ok(Number::Float(l - *r.numer() as f64 / *r.denom() as f64))
            }
            (Number::Rational(l), Number::Fixnum(r)) => {
                Ok(Number::Rational(l - Rational64::from(r)))
            }
            (Number::Rational(l), Number::Float(r)) => {
                Ok(Number::Float(*l.numer() as f64 / *l.denom() as f64 - r))
            }
            (Number::Rational(l), Number::Rational(r)) => Ok(Number::Rational(l - r)),
            (Number::Bignum(l), Number::Fixnum(r)) => Ok(Number::Bignum(l - r)),
            (Number::Bignum(l), Number::Bignum(r)) => Ok(Number::Bignum(l - r)),
            (Number::Bignum(_), Number::Float(_))
            | (Number::Float(_), Number::Bignum(_))
            | (Number::Bignum(_), Number::Rational(_))
            | (Number::Rational(_), Number::Bignum(_)) => Err(RuntimeError::NumberOverflowError),
        }
    }
}

impl Mul for Number {
    type Output = Result<Number>;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Fixnum(l), Number::Fixnum(r)) => {
                if let Some(prod) = l.checked_mul(r) {
                    Ok(Number::Fixnum(prod))
                } else {
                    Ok(Number::Bignum(BigInt::from(l) * r))
                }
            }
            (Number::Fixnum(l), Number::Float(r)) => Ok(Number::Float(l as f64 * r)),
            (Number::Fixnum(l), Number::Rational(r)) => {
                Ok(Number::Rational(Rational64::from(l) * r))
            }
            (Number::Fixnum(l), Number::Bignum(r)) => Ok(Number::Bignum(l * r)),
            (Number::Float(l), Number::Fixnum(r)) => Ok(Number::Float(l * r as f64)),
            (Number::Float(l), Number::Float(r)) => Ok(Number::Float(l * r)),
            (Number::Float(l), Number::Rational(r)) => {
                Ok(Number::Float(l * *r.numer() as f64 / *r.denom() as f64))
            }
            (Number::Rational(l), Number::Fixnum(r)) => {
                Ok(Number::Rational(l * Rational64::from(r)))
            }
            (Number::Rational(l), Number::Float(r)) => {
                Ok(Number::Float(*l.numer() as f64 / *l.denom() as f64 * r))
            }
            (Number::Rational(l), Number::Rational(r)) => Ok(Number::Rational(l * r)),
            (Number::Bignum(l), Number::Fixnum(r)) => Ok(Number::Bignum(l * r)),
            (Number::Bignum(l), Number::Bignum(r)) => Ok(Number::Bignum(l * r)),
            (Number::Bignum(_), Number::Float(_))
            | (Number::Float(_), Number::Bignum(_))
            | (Number::Bignum(_), Number::Rational(_))
            | (Number::Rational(_), Number::Bignum(_)) => Err(RuntimeError::NumberOverflowError),
        }
    }
}

impl Div for Number {
    type Output = Result<Number>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Fixnum(l), Number::Fixnum(r)) => {
                if let Some(quo) = l.checked_div(r) {
                    Ok(Number::Fixnum(quo))
                } else {
                    Ok(Number::Bignum(BigInt::from(l) / r))
                }
            }
            (Number::Fixnum(l), Number::Float(r)) => Ok(Number::Float(l as f64 / r)),
            (Number::Fixnum(l), Number::Rational(r)) => {
                Ok(Number::Rational(Rational64::from(l) / r))
            }
            (Number::Fixnum(l), Number::Bignum(r)) => Ok(Number::Bignum(l / r)),
            (Number::Float(l), Number::Fixnum(r)) => Ok(Number::Float(l / r as f64)),
            (Number::Float(l), Number::Float(r)) => Ok(Number::Float(l / r)),
            (Number::Float(l), Number::Rational(r)) => {
                Ok(Number::Float(l / (*r.numer() as f64 / *r.denom() as f64)))
            }
            (Number::Rational(l), Number::Fixnum(r)) => {
                Ok(Number::Rational(l / Rational64::from(r)))
            }
            (Number::Rational(l), Number::Float(r)) => {
                Ok(Number::Float((*l.numer() as f64 / *l.denom() as f64) / r))
            }
            (Number::Rational(l), Number::Rational(r)) => Ok(Number::Rational(l / r)),
            (Number::Bignum(l), Number::Fixnum(r)) => Ok(Number::Bignum(l / r)),
            (Number::Bignum(l), Number::Bignum(r)) => Ok(Number::Bignum(l / r)),
            (Number::Bignum(_), Number::Float(_))
            | (Number::Float(_), Number::Bignum(_))
            | (Number::Bignum(_), Number::Rational(_))
            | (Number::Rational(_), Number::Bignum(_)) => Err(RuntimeError::NumberOverflowError),
        }
    }
}

impl Rem for Number {
    type Output = Result<Number>;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Number::Fixnum(l), Number::Fixnum(r)) => {
                if let Some(rem) = l.checked_rem(r) {
                    Ok(Number::Fixnum(rem))
                } else {
                    Ok(Number::Bignum(BigInt::from(l) / r))
                }
            }
            (Number::Fixnum(l), Number::Float(r)) => Ok(Number::Float(l as f64 % r)),
            (Number::Fixnum(l), Number::Rational(r)) => {
                Ok(Number::Rational(Rational64::from(l) % r))
            }
            (Number::Fixnum(l), Number::Bignum(r)) => Ok(Number::Bignum(l % r)),
            (Number::Float(l), Number::Fixnum(r)) => Ok(Number::Float(l % r as f64)),
            (Number::Float(l), Number::Float(r)) => Ok(Number::Float(l % r)),
            (Number::Float(l), Number::Rational(r)) => {
                Ok(Number::Float(l % (*r.numer() as f64 / *r.denom() as f64)))
            }
            (Number::Rational(l), Number::Fixnum(r)) => {
                Ok(Number::Rational(l % Rational64::from(r)))
            }
            (Number::Rational(l), Number::Float(r)) => {
                Ok(Number::Float((*l.numer() as f64 / *l.denom() as f64) % r))
            }
            (Number::Rational(l), Number::Rational(r)) => Ok(Number::Rational(l % r)),
            (Number::Bignum(l), Number::Fixnum(r)) => Ok(Number::Bignum(l % r)),
            (Number::Bignum(l), Number::Bignum(r)) => Ok(Number::Bignum(l % r)),
            (Number::Bignum(_), Number::Float(_))
            | (Number::Float(_), Number::Bignum(_))
            | (Number::Bignum(_), Number::Rational(_))
            | (Number::Rational(_), Number::Bignum(_)) => Err(RuntimeError::NumberOverflowError),
        }
    }
}
