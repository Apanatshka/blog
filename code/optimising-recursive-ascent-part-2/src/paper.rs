use crate::{outprod, Error, Iter, State, StackLabel, Parser};
use std::hint::unreachable_unchecked;

/// Use the `Parser` struct to check if that has any influence (it doesn't), as well as inlining
///  `TGoto` since the code is trivial and flipping the matches in `EGoto` to get everything in the
///  same shape for the next variant.
pub fn parse_parser_struct(input: &mut Iter) -> Result<(), Error> {
    use State::*;
    use StackLabel::*;

    let mut p = Parser::default();
    loop {
        match p.label {
            S0 => match input.next() {
                Some('a') => {
                    p.push(SL0);
                    outprod("F = a");
                    outprod("T = F");
                    p.label = S2;
                },
                Some('(') => {
                    p.push(SL0);
                    p.label = S5;
                },
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            }
            S2 => match input.peek() {
                Some('*') => {
                    let _ = input.next();
                    p.label = S7;
                },
                _ => {
                    outprod("E = T");
                    p.label = EGoto
                },
            }
            S5 => match input.next() {
                Some('a') => {
                    p.push(SL5);
                    outprod("F = a");
                    outprod("T = F");
                    p.label = S2;
                }
                Some('(') => {
                    p.push(SL5);
                    p.label = S5; // (self)
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S6 => match input.next() {
                Some('a') => {
                    p.push(SL6);
                    outprod("F = a");
                    p.label = S9;
                }
                Some('(') => {
                    p.push(SL6);
                    p.label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S7 => match input.next() {
                Some('a') => {
                    p.push(SL7);
                    outprod("F = a");
                    p.label = S10
                }
                Some('(') => {
                    p.push(SL7);
                    p.label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
            S9 => match input.peek() {
                Some('*') => {
                    let _ = input.next();
                    p.label = S7;
                }
                _ => {
                    let _ = p.pop(); // 6
                    outprod("E = E + T");
                    p.label = EGoto
                }
            },
            S10 => {
                let _ = p.pop(); // 7
                outprod("T = T * F");
                match p.peek() {
                    SL6 => p.label = S9,
                    _ => p.label = S2,
                }
            }
            EGoto => match input.next() {
                Some('+') => {
                    p.label = S6;
                }
                Some(c@')') => match p.peek() {
                    SL5 => {
                        let _ = p.pop(); // 5
                        outprod("F = ( E )");
                        match p.peek() {
                            SL7 => p.label = S10,
                            _ => {
                                outprod("T = F");
                                match p.peek() {
                                    SL6 => p.label = S9,
                                    _ => p.label = S2,
                                }
                            }
                        }
                    },
                    _ => return Err(Error::Unexpected(c)),
                },
                Some(c) => return Err(Error::Unexpected(c)),
                None => match p.peek() {
                    SL5 => return Err(Error::EOF),
                    _ => {
                        outprod("S = E");
                        return Ok(());
                    }
                },
            },
            TGoto |
            S1 | S3 | S4 | S8 | S11 | FGoto => unsafe { unreachable_unchecked() },
        }
    }
}

/// Make a single match out of it instead of two
pub fn parse_single_match(input: &mut Iter) -> Result<(), Error> {
    use State::*;
    use StackLabel::*;

    let mut p = Parser::default();
    loop {
        match (p.label, input.peek()) {
            (S0, Some('a')) => {
                let _ = input.next();
                p.push(SL0);
                outprod("F = a");
                outprod("T = F");
                p.label = S2;
            },
            (S0, Some('(')) => {
                let _ = input.next();
                p.push(SL0);
                p.label = S5;
            },
            (S0, Some(&c)) => return Err(Error::Unexpected(c)),
            (S0, None) => return Err(Error::EOF),
            (S2, Some('*')) => {
                let _ = input.next();
                p.label = S7;
            },
            (S2, _) => {
                outprod("E = T");
                p.label = EGoto
            },
            (S5, Some('a')) => {
                let _ = input.next();
                p.push(SL5);
                outprod("F = a");
                outprod("T = F");
                p.label = S2;
            }
            (S5, Some('(')) => {
                let _ = input.next();
                p.push(SL5);
                p.label = S5; // (self)
            }
            (S5, Some(&c)) => return Err(Error::Unexpected(c)),
            (S5, None) => return Err(Error::EOF),
            (S6, Some('a')) => {
                let _ = input.next();
                p.push(SL6);
                outprod("F = a");
                p.label = S9;
            }
            (S6, Some('(')) => {
                let _ = input.next();
                p.push(SL6);
                p.label = S5;
            }
            (S6, Some(&c)) => return Err(Error::Unexpected(c)),
            (S6, None) => return Err(Error::EOF),
            (S7, Some('a')) => {
                let _ = input.next();
                p.push(SL7);
                outprod("F = a");
                p.label = S10
            }
            (S7, Some('(')) => {
                let _ = input.next();
                p.push(SL7);
                p.label = S5;
            }
            (S7, Some(&c)) => return Err(Error::Unexpected(c)),
            (S7, None) => return Err(Error::EOF),
            (S9, Some('*')) => {
                let _ = input.next();
                p.label = S7;
            }
            (S9, _) => {
                let _ = p.pop(); // 6
                outprod("E = E + T");
                p.label = EGoto
            }
            (S10, _) => {
                let _ = p.pop(); // 7
                outprod("T = T * F");
                match p.peek() {
                    SL6 => p.label = S9,
                    _ => p.label = S2,
                }
            }
            (EGoto, Some('+')) => {
                let _ = input.next();
                p.label = S6;
            }
            (EGoto, Some(&c@')')) => match p.peek() {
                SL5 => {
                    let _ = input.next();
                    let _ = p.pop(); // 5
                    outprod("F = ( E )");
                    match p.peek() {
                        SL7 => p.label = S10,
                        _ => {
                            outprod("T = F");
                            match p.peek() {
                                SL6 => p.label = S9,
                                _ => p.label = S2,
                            }
                        }
                    }
                },
                _ => return Err(Error::Unexpected(c)),
            },
            (EGoto, Some(&c)) => return Err(Error::Unexpected(c)),
            (EGoto, None) => match p.peek() {
                SL5 => return Err(Error::EOF),
                _ => {
                    outprod("S = E");
                    return Ok(());
                }
            },
            _ => unsafe { unreachable_unchecked() },
        }
    }
}
