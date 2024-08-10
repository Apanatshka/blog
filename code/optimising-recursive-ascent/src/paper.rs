use crate::{outprod, Error, Iter, Sort, State};
use std::hint::unreachable_unchecked;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum State_ {
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
    S0Goto(Sort),
    S5Goto(Sort),
    S6Goto(Sort),
    S7Goto(Sort),
}

pub fn parse(input: &mut Iter) -> Result<(), Error> {
    use State_::*;

    let mut stack = vec![];
    let mut label = S0;
    loop {
        match label {
            S0 => match input.next() {
                Some('a') => {
                    stack.push(0);
                    label = S4;
                }
                Some('(') => {
                    stack.push(0);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S0Goto(sort) => {
                label = match sort {
                    Sort::E => S1,
                    Sort::T => S2,
                    Sort::F => S3,
                }
            }
            S1 => match input.next() {
                Some('+') => {
                    stack.push(1);
                    label = S6;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => {
                    outprod("S = E");
                    return Ok(());
                }
            },
            S2 => match input.peek() {
                Some('*') => {
                    let _ = input.next();
                    stack.push(2);
                    label = S7;
                }
                _ => {
                    outprod("E = T");
                    label = match stack.last().unwrap() {
                        0 => S0Goto(Sort::E),
                        5 => S5Goto(Sort::E),
                        _ => unreachable!(),
                    }
                }
            },
            S3 => {
                outprod("T = F");
                label = match stack.last().unwrap() {
                    0 => S0Goto(Sort::T),
                    5 => S5Goto(Sort::T),
                    6 => S6Goto(Sort::T),
                    _ => unreachable!(),
                }
            }
            S4 => {
                outprod("F = a");
                label = match stack.last().unwrap() {
                    0 => S0Goto(Sort::F),
                    5 => S5Goto(Sort::F),
                    6 => S6Goto(Sort::F),
                    7 => S7Goto(Sort::F),
                    _ => unreachable!(),
                }
            }
            S5 => match input.next() {
                Some('a') => {
                    stack.push(5);
                    label = S4;
                }
                Some('(') => {
                    stack.push(5);
                    label = S5; // (self)
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S5Goto(sort) => {
                label = match sort {
                    Sort::E => S8,
                    Sort::T => S2,
                    Sort::F => S3,
                }
            }
            S6 => match input.next() {
                Some('a') => {
                    stack.push(6);
                    label = S4;
                }
                Some('(') => {
                    stack.push(6);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S6Goto(sort) => {
                label = match sort {
                    Sort::T => S9,
                    Sort::F => S3,
                    _ => unreachable!(),
                }
            }
            S7 => match input.next() {
                Some('a') => {
                    stack.push(7);
                    label = S4;
                }
                Some('(') => {
                    stack.push(7);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S7Goto(sort) => {
                debug_assert!(sort == Sort::F);
                label = S10;
            }
            S8 => match input.next() {
                Some('+') => {
                    stack.push(8);
                    label = S6;
                }
                Some(')') => {
                    stack.push(8);
                    label = S11;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S9 => match input.peek() {
                Some('*') => {
                    let _ = input.next();
                    stack.push(9);
                    label = S7;
                }
                _ => {
                    let _ = stack.pop(); // 6
                    let _ = stack.pop(); // 1 or 8
                    outprod("E = E + T");
                    label = match stack.last().unwrap() {
                        0 => S0Goto(Sort::E),
                        5 => S5Goto(Sort::E),
                        _ => unreachable!(),
                    }
                }
            },
            S10 => {
                let _ = stack.pop(); // 7
                let _ = stack.pop(); // 2 or 9
                outprod("T = T * F");
                label = match stack.last().unwrap() {
                    0 => S0Goto(Sort::T),
                    5 => S5Goto(Sort::T),
                    6 => S6Goto(Sort::T),
                    _ => unreachable!(),
                }
            }
            S11 => {
                let _ = stack.pop(); // 8
                let _ = stack.pop(); // 5
                outprod("F = ( E )");
                label = match stack.last().unwrap() {
                    0 => S0Goto(Sort::F),
                    5 => S5Goto(Sort::F),
                    6 => S6Goto(Sort::F),
                    7 => S7Goto(Sort::F),
                    _ => unreachable!(),
                }
            }
        }
    }
}

/// Note the Goto is now labeled with the sort, and checks the stack for the state
///   (instead of the other way)
pub fn parse_reverse_goto(input: &mut Iter) -> Result<(), Error> {
    use State::*;

    let mut stack = vec![];
    let mut label = S0;
    loop {
        match label {
            S0 => match input.next() {
                Some('a') => {
                    stack.push(0);
                    label = S4;
                }
                Some('(') => {
                    stack.push(0);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S1 => match input.next() {
                Some('+') => {
                    stack.push(1);
                    label = S6;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => {
                    outprod("S = E");
                    return Ok(());
                }
            },
            S2 => match input.peek() {
                Some('*') => {
                    let _ = input.next();
                    stack.push(2);
                    label = S7;
                }
                _ => {
                    outprod("E = T");
                    label = EGoto
                }
            },
            S3 => {
                outprod("T = F");
                label = TGoto
            }
            S4 => {
                outprod("F = a");
                label = FGoto
            }
            S5 => match input.next() {
                Some('a') => {
                    stack.push(5);
                    label = S4;
                }
                Some('(') => {
                    stack.push(5);
                    label = S5; // (self)
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S6 => match input.next() {
                Some('a') => {
                    stack.push(6);
                    label = S4;
                }
                Some('(') => {
                    stack.push(6);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S7 => match input.next() {
                Some('a') => {
                    stack.push(7);
                    label = S4;
                }
                Some('(') => {
                    stack.push(7);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S8 => match input.next() {
                Some('+') => {
                    stack.push(8);
                    label = S6;
                }
                Some(')') => {
                    stack.push(8);
                    label = S11;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S9 => match input.peek() {
                Some('*') => {
                    let _ = input.next();
                    stack.push(9);
                    label = S7;
                }
                _ => {
                    let _ = stack.pop(); // 6
                    let _ = stack.pop(); // 1
                    outprod("E = E + T");
                    label = EGoto
                }
            },
            S10 => {
                let _ = stack.pop(); // 7
                let _ = stack.pop(); // 2 or 9
                outprod("T = T * F");
                label = TGoto
            }
            S11 => {
                let _ = stack.pop(); // 8
                let _ = stack.pop(); // 5
                outprod("F = ( E )");
                label = FGoto
            }
            EGoto => match stack[stack.len() - 1] {
                5 => label = S8,
                _ => label = S1,
            },
            TGoto => match stack[stack.len() - 1] {
                6 => label = S9,
                _ => label = S2,
            },
            FGoto => match stack[stack.len() - 1] {
                7 => label = S10,
                _ => label = S3,
            },
        }
    }
}

/// Note that S3 and S4 (which were just jumps) were inlined, and then any gotos were inlined
pub fn parse_chain_elim(input: &mut Iter) -> Result<(), Error> {
    use State::*;

    let mut stack = vec![];
    let mut label = S0;
    loop {
        match label {
            S0 => match input.next() {
                Some('a') => {
                    stack.push(0);
                    outprod("F = a");
                    outprod("T = F");
                    label = S2;
                }
                Some('(') => {
                    stack.push(0);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S1 => match input.next() {
                Some('+') => {
                    stack.push(1);
                    label = S6;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => {
                    outprod("S = E");
                    return Ok(());
                }
            },
            S2 => match input.peek() {
                Some('*') => {
                    let _ = input.next();
                    stack.push(2);
                    label = S7;
                }
                _ => {
                    outprod("E = T");
                    label = EGoto
                }
            },
            S5 => match input.next() {
                Some('a') => {
                    stack.push(5);
                    outprod("F = a");
                    outprod("T = F");
                    label = S2;
                }
                Some('(') => {
                    stack.push(5);
                    label = S5; // (self)
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S6 => match input.next() {
                Some('a') => {
                    stack.push(6);
                    outprod("F = a");
                    outprod("T = F");
                    label = S9;
                }
                Some('(') => {
                    stack.push(6);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S7 => match input.next() {
                Some('a') => {
                    stack.push(7);
                    outprod("F = a");
                    label = S10;
                }
                Some('(') => {
                    stack.push(7);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S8 => match input.next() {
                Some('+') => {
                    stack.push(8);
                    label = S6;
                }
                Some(')') => {
                    stack.push(8);
                    label = S11;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S9 => match input.peek() {
                Some('*') => {
                    let _ = input.next();
                    stack.push(9);
                    label = S7;
                }
                _ => {
                    let _ = stack.pop(); // 6
                    let _ = stack.pop(); // 1
                    outprod("E = E + T");
                    label = EGoto
                }
            },
            S10 => {
                let _ = stack.pop(); // 7
                let _ = stack.pop(); // 2 or 9
                outprod("T = T * F");
                label = TGoto
            }
            S11 => {
                let _ = stack.pop(); // 8
                let _ = stack.pop(); // 5
                outprod("F = ( E )");
                label = FGoto
            }
            EGoto => match stack[stack.len() - 1] {
                5 => label = S8,
                _ => label = S1,
            },
            TGoto => match stack[stack.len() - 1] {
                6 => label = S9,
                _ => label = S2,
            },
            FGoto => match stack[stack.len() - 1] {
                7 => label = S10,
                _ => {
                    outprod("T = F");
                    label = TGoto
                }
            },
            S3 | S4 => unsafe { unreachable_unchecked() },
        }
    }
}

/// Note we don't to push stack numbers that are unconditionally popped and can never be viewed in
///  the gotos (1/2/8/9). We do keep every stack number that's specifically branched on in the gotos
///  (5/6/7). We could eliminate stack number 0 in theory according to the push graph minimal vertex
///  cover, but then the stack might be empty in some situation where we want to view the top of the
///  stack.
pub fn parse_minpush(input: &mut Iter) -> Result<(), Error> {
    use State::*;

    let mut stack = vec![];
    let mut label = S0;
    loop {
        match label {
            S0 => match input.next() {
                Some('a') => {
                    stack.push(0);
                    outprod("F = a");
                    outprod("T = F");
                    label = S2;
                }
                Some('(') => {
                    stack.push(0);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S1 => match input.next() {
                Some('+') => {
                    label = S6;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => {
                    outprod("S = E");
                    return Ok(());
                }
            },
            S2 => match input.peek() {
                Some('*') => {
                    let _ = input.next();
                    label = S7;
                }
                _ => {
                    outprod("E = T");
                    label = EGoto
                }
            },
            S5 => match input.next() {
                Some('a') => {
                    stack.push(5);
                    outprod("F = a");
                    outprod("T = F");
                    label = S2;
                }
                Some('(') => {
                    stack.push(5);
                    label = S5; // (self)
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S6 => match input.next() {
                Some('a') => {
                    stack.push(6);
                    outprod("F = a");
                    outprod("T = F");
                    label = S9;
                }
                Some('(') => {
                    stack.push(6);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S7 => match input.next() {
                Some('a') => {
                    stack.push(7);
                    outprod("F = a");
                    label = S10;
                }
                Some('(') => {
                    stack.push(7);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S8 => match input.next() {
                Some('+') => {
                    label = S6;
                }
                Some(')') => {
                    label = S11;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S9 => match input.peek() {
                Some('*') => {
                    let _ = input.next();
                    label = S7;
                }
                _ => {
                    let _ = stack.pop(); // 6
                    outprod("E = E + T");
                    label = EGoto
                }
            },
            S10 => {
                let _ = stack.pop(); // 7
                outprod("T = T * F");
                label = TGoto
            }
            S11 => {
                let _ = stack.pop(); // 5
                outprod("F = ( E )");
                label = FGoto
            }
            EGoto => match stack[stack.len() - 1] {
                5 => label = S8,
                _ => label = S1,
            },
            TGoto => match stack[stack.len() - 1] {
                6 => label = S9,
                _ => label = S2,
            },
            FGoto => match stack[stack.len() - 1] {
                7 => label = S10,
                _ => {
                    outprod("T = F");
                    label = TGoto
                }
            },
            S3 | S4 => unsafe { unreachable_unchecked() },
        }
    }
}

/// Now we can inline all labels used only once. We go from 2 unused labels to 6, with 9 labels left
///   in use.
pub fn parse_max_inline(input: &mut Iter) -> Result<(), Error> {
    use State::*;

    let mut stack = vec![];
    let mut label = S0;
    loop {
        match label {
            S0 => match input.next() {
                Some('a') => {
                    stack.push(0);
                    outprod("F = a");
                    outprod("T = F");
                    label = S2;
                }
                Some('(') => {
                    stack.push(0);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S2 => match input.peek() {
                Some('*') => {
                    let _ = input.next();
                    label = S7;
                }
                _ => {
                    outprod("E = T");
                    label = EGoto
                }
            },
            S5 => match input.next() {
                Some('a') => {
                    stack.push(5);
                    outprod("F = a");
                    outprod("T = F");
                    label = S2;
                }
                Some('(') => {
                    stack.push(5);
                    label = S5; // (self)
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S6 => match input.next() {
                Some('a') => {
                    stack.push(6);
                    outprod("F = a");
                    label = S9;
                }
                Some('(') => {
                    stack.push(6);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S7 => match input.next() {
                Some('a') => {
                    stack.push(7);
                    outprod("F = a");
                    label = S10
                }
                Some('(') => {
                    stack.push(7);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S9 => match input.peek() {
                Some('*') => {
                    let _ = input.next();
                    label = S7;
                }
                _ => {
                    let _ = stack.pop(); // 6
                    outprod("E = E + T");
                    label = EGoto
                }
            },
            S10 => {
                let _ = stack.pop(); // 7
                outprod("T = T * F");
                label = TGoto
            }
            EGoto => match stack[stack.len() - 1] {
                5 => match input.next() {
                    Some('+') => {
                        label = S6;
                    }
                    Some(')') => {
                        let _ = stack.pop(); // 5
                        outprod("F = ( E )");
                        match stack[stack.len() - 1] {
                            7 => label = S10,
                            _ => {
                                outprod("T = F");
                                label = TGoto
                            }
                        }
                    }
                    Some(c) => return Err(Error::Unexpected(c)),
                    None => return Err(Error::EOF),
                },
                _ => match input.next() {
                    Some('+') => {
                        label = S6;
                    }
                    Some(c) => return Err(Error::Unexpected(c)),
                    None => {
                        outprod("S = E");
                        return Ok(());
                    }
                },
            },
            TGoto => match stack[stack.len() - 1] {
                6 => label = S9,
                _ => label = S2,
            },
            S1 | S3 | S4 | S8 | S11 | FGoto => unsafe { unreachable_unchecked() },
        }
    }
}
