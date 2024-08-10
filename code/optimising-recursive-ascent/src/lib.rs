pub mod paper;

use std::iter::Peekable;
use std::str::Chars;

/*
S = E
E = E + T
E = T
T = T * F
T = F
F = a
F = ( E )
 */

pub type Iter<'a> = Peekable<Chars<'a>>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Sort {
    // S,
    E,
    T,
    F,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum State {
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    EGoto,
    TGoto,
    FGoto,
}

#[inline(never)]
fn outprod(rule: &str) {
    // a semantic action
    // eprintln!("{}", rule)
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Error {
    EOF,
    Unexpected(char),
}