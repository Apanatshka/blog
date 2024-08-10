pub mod parser;
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
fn outprod(_rule: &str) {
    // a semantic action
    // eprintln!("{}", rule)
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Error {
    EOF,
    Unexpected(char),
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum StackLabel {
    SL0,
    SL5,
    SL6,
    SL7,
}

pub struct Parser {
    stack: Vec<StackLabel>,
    stack_last: StackLabel,
    pub label: State,
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            stack: vec![],
            stack_last: StackLabel::SL0,
            label: State::S0,
        }
    }
}

impl Parser {
    #[inline(always)]
    fn push(&mut self, n: StackLabel) {
        self.stack.push(self.stack_last);
        self.stack_last = n;
    }

    #[inline(always)]
    fn pop(&mut self) {
        self.stack_last = unsafe { self.stack.pop().unwrap_unchecked() };
    }

    #[inline(always)]
    fn peek(&self) -> StackLabel {
        self.stack_last
    }
}