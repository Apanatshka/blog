use crate::{outprod, Error, Iter, State, StackLabel, Parser};
use std::hint::unreachable_unchecked;

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

/// We're starting with reversed goto again, that seems nice. But now we do ascent-descent, where
///   we switch to LL when committing to a rule. This inlines S10 and S11 instead of S3 and S4.
pub fn parse_asc_desc(input: &mut Iter) -> Result<(), Error> {
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
                    let _ = stack.pop(); // 5
                    outprod("F = ( E )");
                    label = FGoto
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
            EGoto => match stack[stack.len() - 1] {
                5 => label = S8,
                _ => label = S1,
            },
            TGoto => match stack[stack.len() - 1] {
                6 => label = S9,
                _ => label = S2,
            },
            FGoto => match stack[stack.len() - 1] {
                7 => {
                    let _ = stack.pop(); // 7
                    let _ = stack.pop(); // 2 or 9
                    outprod("T = T * F");
                    label = TGoto
                }
                _ => label = S3,
            },
            S10 | S11 => unsafe { unreachable_unchecked() },
        }
    }
}

/// If you push before going to the state, states that have the same "T-Table" (shift/reduce
///   actions) can be 'merged', at least in label (S0/S5/S6/S7). Now if you do the recursive ascent
///   inlining of states 10/11, you've lost the static information on where you are and cannot
///   inline the Goto label. But we did get rid of S5/S6/S7. So which one is better? Probably the
///   one that pushes late, because it pushes less... Pushing less? We know a trick for that.
pub fn parse_push_first(input: &mut Iter) -> Result<(), Error> {
    use State::*;

    let mut stack = vec![0];
    let mut label = S0;
    loop {
        match label {
            S0 => match input.next() {
                Some('a') => {
                    stack.push(4);
                    label = S4;
                }
                Some('(') => {
                    stack.push(5);
                    label = S0;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S1 => match input.next() {
                Some('+') => {
                    stack.push(6);
                    label = S0;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => {
                    outprod("S = E");
                    return Ok(());
                }
            },
            S2 => match input.peek() {
                Some('*') => {
                    let _ = input.next(); // *
                    stack.push(7);
                    label = S0;
                }
                _ => {
                    let _ = stack.pop(); // 2
                    outprod("E = T");
                    label = EGoto
                }
            },
            S3 => {
                let _ = stack.pop(); // 3
                outprod("T = F");
                label = TGoto
            }
            S4 => {
                let _ = stack.pop(); // 4
                outprod("F = a");
                label = FGoto
            }
            S8 => match input.next() {
                Some('+') => {
                    stack.push(6);
                    label = S0;
                }
                Some(')') => {
                    let _ = stack.pop(); // 8
                    let _ = stack.pop(); // 5
                    outprod("F = ( E )");
                    label = FGoto
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S9 => match input.peek() {
                Some('*') => {
                    let _ = input.next(); // *
                    stack.push(7);
                    label = S0;
                }
                _ => {
                    let _ = stack.pop(); // 9
                    let _ = stack.pop(); // 6
                    let _ = stack.pop(); // 1
                    outprod("E = E + T");
                    label = EGoto
                }
            },
            S10 => {
                let _ = stack.pop(); // 10
                let _ = stack.pop(); // 7
                let _ = stack.pop(); // 2 or 9
                outprod("T = T * F");
                label = TGoto
            },
            S11 => {
                let _ = stack.pop(); // 11
                let _ = stack.pop(); // 8
                let _ = stack.pop(); // 5
                outprod("F = ( E )");
                label = FGoto
            },
            EGoto => {
                match stack[stack.len() - 1] {
                    5 => {
                        stack.push(8);
                        label = S8
                    }
                    // 0
                    _ => {
                        stack.push(1);
                        label = S1
                    }
                }
            }
            TGoto => match stack[stack.len() - 1] {
                6 => {
                    stack.push(9);
                    label = S9
                }
                _ => {
                    stack.push(2);
                    label = S2
                }
            },
            FGoto => match stack[stack.len() - 1] {
                7 => {
                    stack.push(10);
                    label = S10
                }
                _ => {
                    stack.push(3);
                    label = S3
                }
            },
            S5 | S6 | S7 => unsafe { unreachable_unchecked() },
        }
    }
}

/// So we continue from push_first with a minpush approach now: push 0/5/6/7. This leaves us once
///   more with minimal pushing to the stack.
pub fn parse_minpush(input: &mut Iter) -> Result<(), Error> {
    use State::*;

    let mut stack = vec![0];
    let mut label = S0;
    loop {
        match label {
            S0 => match input.next() {
                Some('a') => {
                    label = S4;
                }
                Some('(') => {
                    stack.push(5);
                    label = S0;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S1 => match input.next() {
                Some('+') => {
                    stack.push(6);
                    label = S0;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => {
                    outprod("S = E");
                    return Ok(());
                }
            },
            S2 => match input.peek() {
                Some('*') => {
                    let _ = input.next(); // *
                    stack.push(7);
                    label = S0;
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
            S8 => match input.next() {
                Some('+') => {
                    stack.push(6);
                    label = S0;
                }
                Some(')') => {
                    let _ = stack.pop(); // 5
                    outprod("F = ( E )");
                    label = FGoto
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S9 => match input.peek() {
                Some('*') => {
                    let _ = input.next(); // *
                    stack.push(7);
                    label = S0;
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
            },
            S11 => {
                let _ = stack.pop(); // 5
                outprod("F = ( E )");
                label = FGoto
            },
            EGoto => {
                match stack[stack.len() - 1] {
                    5 => label = S8,
                    // 0
                    _ => label = S1,
                }
            }
            TGoto => match stack[stack.len() - 1] {
                6 => label = S9,
                _ => label = S2,
            },
            FGoto => match stack[stack.len() - 1] {
                7 => label = S10,
                _ => label = S3,
            },
            S5 | S6 | S7 => unsafe { unreachable_unchecked() },
        }
    }
}

/// Now with nothing left to do, we inline every label used in only one place. We go from 3 unused
///   labels to 11, only 4 labels left in use!
pub fn parse_inline1(input: &mut Iter) -> Result<(), Error> {
    use State::*;

    let mut stack = vec![0];
    let mut label = S0;
    loop {
        match label {
            S0 => match input.next() {
                Some('a') => {
                    outprod("F = a");
                    label = FGoto
                }
                Some('(') => {
                    stack.push(5);
                    label = S0;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            EGoto => {
                match stack[stack.len() - 1] {
                    5 => {
                        match input.next() {
                            Some('+') => {
                                stack.push(6);
                                label = S0;
                            }
                            Some(')') => {
                                let _ = stack.pop(); // 5
                                outprod("F = ( E )");
                                label = FGoto
                            }
                            Some(c) => return Err(Error::Unexpected(c)),
                            None => return Err(Error::EOF),
                        }
                    }
                    _ => {
                        // assert!(stack[stack.len() - 1] == 0)
                        match input.next() {
                            Some('+') => {
                                stack.push(6);
                                label = S0;
                            }
                            Some(c) => return Err(Error::Unexpected(c)),
                            None => {
                                outprod("S = E");
                                return Ok(());
                            }
                        }
                    }
                }
            }
            TGoto => match stack[stack.len() - 1] {
                6 => match input.peek() {
                    Some('*') => {
                        let _ = input.next(); // *
                        stack.push(7);
                        label = S0;
                    }
                    _ => {
                        let _ = stack.pop(); // 6
                        outprod("E = E + T");
                        label = EGoto
                    }
                },
                _ => match input.peek() {
                    Some('*') => {
                        let _ = input.next(); // *
                        stack.push(7);
                        label = S0;
                    }
                    _ => {
                        outprod("E = T");
                        label = EGoto
                    }
                },
            },
            FGoto => match stack[stack.len() - 1] {
                7 => {
                    let _ = stack.pop(); // 7
                    outprod("T = T * F");
                    label = TGoto
                }
                _ => {
                    outprod("T = F");
                    label = TGoto
                }
            },
            S1 | S2 | S3 | S4 | S5 | S6 | S7 | S8 | S9 | S10 | S11 => unsafe {
                unreachable_unchecked()
            },
        }
    }
}

/// Then with a nested match exchange and factoring out a label assignment, we see another two
///   labels used in only one place, and go down to two labels.
/// If you're willing to duplicate the semantic actions (calls to outprod are placeholders for
///   these), you can do a single match on the `input.next()` result. Probably not worth it.
pub fn parse_inline2(input: &mut Iter) -> Result<(), Error> {
    use State::*;

    let mut stack = vec![0];
    let mut label = S0;
    loop {
        match label {
            S0 => match input.next() {
                Some('a') => {
                    outprod("F = a");
                    label = FGoto
                }
                Some('(') => {
                    stack.push(5);
                    label = S0;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            FGoto => {
                match stack[stack.len() - 1] {
                    7 => {
                        let _ = stack.pop(); // 7
                        outprod("T = T * F");
                    }
                    _ => {
                        outprod("T = F");
                    }
                }
                match input.peek() {
                    Some('*') => {
                        let _ = input.next(); // *
                        stack.push(7);
                        label = S0;
                    }
                    _ => {
                        match stack[stack.len() - 1] {
                            6 => {
                                let _ = stack.pop(); // 6
                                outprod("E = E + T");
                            }
                            _ => {
                                outprod("E = T");
                            }
                        }
                        match stack[stack.len() - 1] {
                            5 => {
                                match input.next() {
                                    Some('+') => {
                                        stack.push(6);
                                        label = S0;
                                    }
                                    Some(')') => {
                                        let _ = stack.pop(); // 5
                                        outprod("F = ( E )");
                                        label = FGoto // (self)
                                    }
                                    Some(c) => return Err(Error::Unexpected(c)),
                                    None => return Err(Error::EOF),
                                }
                            }
                            _ => {
                                // assert!(stack[stack.len() - 1] == 0)
                                match input.next() {
                                    Some('+') => {
                                        stack.push(6);
                                        label = S0;
                                    }
                                    Some(c) => return Err(Error::Unexpected(c)),
                                    None => {
                                        outprod("S = E");
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            S1 | S2 | S3 | S4 | S5 | S6 | S7 | S8 | S9 | S10 | S11 | EGoto | TGoto => unsafe {
                unreachable_unchecked()
            },
        }
    }
}

pub fn parse_single_input_next1(input: &mut Iter) -> Result<(), Error> {
    use StackLabel::*;
    use State::*;

    let mut stack = vec![SL0];
    let mut label = S0;
    loop {
        match label {
            S0 => match input.next() {
                Some('a') => {
                    outprod("F = a");
                    label = FGoto
                }
                Some('(') => {
                    stack.push(SL5);
                    label = S0;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            FGoto => {
                match stack[stack.len() - 1] {
                    SL7 => {
                        let _ = stack.pop(); // 7
                        outprod("T = T * F");
                    }
                    _ => {
                        outprod("T = F");
                    }
                }
                match input.next() {
                    Some('*') => {
                        stack.push(SL7);
                        label = S0;
                    }
                    i => {
                        match stack[stack.len() - 1] {
                            SL6 => {
                                let _ = stack.pop(); // 6
                                outprod("E = E + T");
                            }
                            _ => {
                                outprod("E = T");
                            }
                        }
                        match i {
                            Some('+') => {
                                stack.push(SL6);
                                label = S0;
                            }
                            Some(c @ ')') => {
                                match stack[stack.len() - 1] {
                                    SL5 => {
                                        let _ = stack.pop(); // 5
                                        outprod("F = ( E )");
                                        label = FGoto // (self)
                                    }
                                    _ => return Err(Error::Unexpected(c)),
                                }
                            }
                            Some(c) => return Err(Error::Unexpected(c)),
                            None => {
                                return match stack[stack.len() - 1] {
                                    SL5 => Err(Error::EOF),
                                    _ => {
                                        outprod("S = E");
                                        Ok(())
                                    }
                                }
                            }
                        }
                    }
                }
            }
            S1 | S2 | S3 | S4 | S5 | S6 | S7 | S8 | S9 | S10 | S11 | EGoto | TGoto => unsafe {
                unreachable_unchecked()
            },
        }
    }
}

pub fn parse_single_input_next(input: &mut Iter) -> Result<(), Error> {
    use StackLabel::*;
    use State::*;

    let mut p = Parser::default();

    loop {
        match (p.label, input.next()) {
            (S0, Some('a')) => {
                outprod("F = a");
                p.label = FGoto
            }
            (S0, Some('(')) => {
                p.push(SL5);
                // p.label = S0 // (self)
            }
            (S0, Some(c)) => return Err(Error::Unexpected(c)),
            (S0, None) => return Err(Error::EOF),
            (FGoto, Some('*')) => {
                if let SL7 = p.peek() {
                    outprod("T = T * F");
                } else {
                    outprod("T = F");
                    p.push(SL7);
                }
                p.label = S0
            }
            (FGoto, Some('+')) => {
                if let SL7 = p.peek() {
                    p.pop(); // 7
                    outprod("T = T * F");
                } else {
                    outprod("T = F");
                }
                if let SL6 = p.peek() {
                    outprod("E = E + T");
                } else {
                    outprod("E = T");
                    p.push(SL6);
                }
                p.label = S0;
            }
            (FGoto, Some(c @ ')')) => {
                if let SL7 = p.peek() {
                    p.pop(); // 7
                    outprod("T = T * F");
                } else {
                    outprod("T = F");
                }
                if let SL6 = p.peek() {
                    p.pop(); // 6
                    outprod("E = E + T");
                } else {
                    outprod("E = T");
                }
                if let SL5 = p.peek() {
                    p.pop(); // 5
                    outprod("F = ( E )");
                    // p.label = FGoto // (self)
                } else {
                    return Err(Error::Unexpected(c));
                }
            }
            (FGoto, Some(c)) => return Err(Error::Unexpected(c)),
            (FGoto, None) => {
                if let SL7 = p.peek() {
                    p.pop(); // 7
                    outprod("T = T * F");
                } else {
                    outprod("T = F");
                }
                if let SL6 = p.peek() {
                    p.pop(); // 6
                    outprod("E = E + T");
                } else {
                    outprod("E = T");
                }
                return if let SL5 = p.peek() {
                    Err(Error::EOF)
                } else {
                    outprod("S = E");
                    Ok(())
                };
            }
            _ => unsafe { unreachable_unchecked() },
        }
    }
}
