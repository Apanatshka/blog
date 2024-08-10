+++
title = "Optimising Recursive Ascent Parsing, Part 2"
date = "2024-08-10"
taxonomies.tags = ["theory of computation", "automata", "nfa", "dfa", "rust"]
+++

Welcome back! Previously, on [Optimising Recursive Ascent Parsing](@/optimising-recursive-ascent.md), we explored the ideas from [a 1990 paper called _Optimizing Directly Executable LR Parsers_ by Peter Pfahler](https://doi.org/10.1007/3-540-53669-8_82). With the paper's example grammar, and the described optimisations, we managed to optimise away 6 out of 15 states in the parser. But that's peanuts compared to what we'll do in this post! We'll be taking inspiration from another 1990 paper, this one is called [_Even Faster LR Parsing_ by Nigel Horspool and Michael Whitney](https://doi.org/10.1002/spe.4380200602). The optimisations make the recursive ascent parser significantly smaller, keeping only 4 out of the original 15 states. However, the optimisation steps are not always a performance win on our little test input...

## Quick recap

Here's the example grammar again. The grammar is a simple arithmetic grammar that has been made unambiguous by encoding the precedent relation between multiplication and addition (multiplication binds tighter):

| | |
:- | :-
$S = E$     | (1)
$E = E + T$ | (2)
$E = T$     | (3)
$T = T * F$ | (4)
$T = F$     | (5)
$F = a$     | (6)
$F = ( E )$ | (7)

The LALR(1) parse table for this grammar is the following:

<div class="parsetable">

|          | `a`  | `+`  | `*`  | `(`  | `)`   | `$`  | | `E` | `T` | `F` |
|:---------|:----:|:----:|:----:|:----:|:-----:|:----:|-|:---:|:---:|:---:|
| **S0**   | s S4 |      |      | s S5 |       |      | | S1  | S2  | S3  |
| **S1**   |      | s S6 |      |      |       | ra 1 | |     |     |     |
| **S2**   |      | r 3  | s S7 |      | r 3   | r 3  | |     |     |     |
| **S3**   |      | r 5  | r 5  |      | r 5   | r 5  | |     |     |     |
| **S4**   |      | r 6  | r 6  |      | r 6   | r 6  | |     |     |     |
| **S5**   | s S4 |      |      | s S5 |       |      | | S8  | S2  | S3  |
| **S6**   | s S4 |      |      | s S5 |       |      | |     | S9  | S3  |
| **S7**   | s S4 |      |      | s S5 |       |      | |     |     | S10 |
| **S8**   |      | s S6 |      |      | s S11 |      | |     |     |     |
| **S9**   |      | r 2  | s S7 |      | r 2   | r 2  | |     |     |     |
| **S10**  |      | r 4  | r 4  |      | r 4   | r 4  | |     |     |     |
| **S11**  |      | r 7  | r 7  |      | r 7   | r 7  | |     |     |     |
</div>

Note that I've fused reducing by rule 1 (our only rule of the start symbol) with accepting the input (`ra` = reduce + accept). This way we also don't need a column `S` in the goto part of the table.

I'll repeat the definitions that we're still using the code:

```rust
pub type Iter<'a> = Peekable<Chars<'a>>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Sort { /*   S,*/    E,    T,    F }

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum State {    S0,    S1,    S2,    S3,    S4,    S5,    S6,    S7,    S8,    S9,    S10,    S11,    EGoto,    TGoto,    FGoto }

#[inline(never)]
fn outprod(rule: &str) {
    // a semantic action
    eprintln!("{}", rule)
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Error {    EOF,    Unexpected(char) }
```

So we have our input type `Iter` of characters we can peek into without consuming for that single lookahead. We've got `Sort`s, a `State` enum. Note how the states are just `S#`, but the goto actions will be handled by sort instead of by state. This is what we need for the _reverse goto_ optimisation of Pfahler's, which we will use again in this post. To do a "semantic action" when reducing we mark these places with an uninlinable `outprod`. This will just be a placeholder for an expensive operation. There's an `Error` type, no surprises here. 

Let's continue to the _reverse goto_ recursive ascent code for the parse table. You don't have to read through and study all of it, have a quick look, then I'll highlight some insights after:

```rust
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
                    stack.push(1);
                    stack.pop();
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
                    stack.push(2);
                    stack.pop();
                    outprod("E = T");
                    label = EGoto
                }
            },
            S3 => {
                stack.push(3);
                stack.pop();
                outprod("T = F");
                label = TGoto
            }
            S4 => {
                stack.push(4);
                stack.pop();
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
                    stack.push(9);
                    let _ = stack.pop(); // 9
                    let _ = stack.pop(); // 6
                    let _ = stack.pop(); // 1
                    outprod("E = E + T");
                    label = EGoto
                }
            },
            S10 => {
                stack.push(10);
                let _ = stack.pop(); // 10
                let _ = stack.pop(); // 7
                let _ = stack.pop(); // 2 or 9
                outprod("T = T * F");
                label = TGoto
            }
            S11 => {
                stack.push(11);
                let _ = stack.pop(); // 11
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
```

We use a mutable variable `label` to hold the current `State` of the parser. Since this label is assigned static values throughout the code, this and the loop/match gets compiled away into goto instructions and labels by the compiler. The `stack` is used to keep track of the state numbers in which we pushed a terminal or did the goto on a sort. So a shift action pushes the current state number and sets `label` to the next `State`. A reduce action pops off all but one state number on the `stack` and uses the produced sort to decide the `label` with the form `SortGoto`. This goto `State` handles the shared logic of jumping to the next state based on the state number on the `stack`.

In the previous post the next optimisation we applied was Chain Elimination, which inlined all the states that did (only) a reduce on rules with a single symbol on the right-hand side (RHS). This eliminated `S3` and `S4`, but my gripe with it is that it duplicates the code of `S4` in four places. And remember that `outprod("F = a")` is supposed to represent a significant amount of code in both size and execution cost. Pfahler argues in his paper that this is a very effective optimisation as most rules with a single symbol RHS do not have an expensive associated semantic action. This makes sense for rules 1, 3, and 5 of the grammar, which embed one sort into another sort for the purpose of encoding the precedence relation between $+$ and $*$. But rule 6, $F = a$ is a leaf node in the tree. I think it's only natural for that rule to be observed, and yet by Chain Elimination we inline its semantic action all over the code. So let's do something else...

## Code Sharing Through Push-First

Notice how the left part of the parse table (the shift/reduce part) has a couple of duplicate rows. `S0`, `S5`, `S6`, and `S7` have almost the same code due to this. Our guiding paper for this post says we should be able to do code sharing between these states but doesn't exactly spell out the trick to that. The issue that makes the code not quite the same is that we push the current stack number. The trick to this that isn't mentioned in the paper, or at least my trick, is to push a state number _before_ you go to the state. This means that at the start of our program we need to have the start state zero already on the stack:

```rust
pub fn parse_push_first(input: &mut Iter) -> Result<(), Error> {
    use State::*;

    let mut stack = vec![0];
    let mut label = S0;
```

State `S0` now doesn't push its own number on the stack, but the number of the state that it's going to next:

```rust
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
```

Note how we push `4` before setting the `label` to `S4` to continue there, and we push `5` as before, but now we're going to state `S0`. This is because we've gotten rid of `S5` entirely and use `S0` instead now. Because we push the state number first, we can still distinguish our code-shared states.

### More Code Sharing Among Similar Enough States?

If you have two states with the same left side of the parse table for most but not all columns, you can share the code for that same part by testing for only the different part in each state, and have a new label for the shared part. According to the paper, in practice there are commonly many states that share a core set of inputs/actions that can be shared. They suggest using a bitvector per state to test for an input where that state should use the shared core. If the input matches the bit in that vector, you jump to the label for the shared core. With and without the bitvector, this is apparently very effective at reducing the generated code size with very little cost to the run time performance.

Speaking of, let's check that for our simple push-first trick:

```
parse_reverse_goto/a+a*(a+a)*a
                        time:   [138.27 ns 138.47 ns 138.65 ns]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

parse_push_first/a+a*(a+a)*a
                        time:   [210.75 ns 210.97 ns 211.18 ns]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
```

That is a lot more performance degradation than I expected... Note that these measurements are kinda dumb, you should take them with a grain of salt. This is using a tiny example grammar, a single tiny input, and running the benchmarks on a desktop that is probably doing some other background tasks too.  
Let's see if we can't improve the original `reverse_goto` time by applying more optimisations.

## Minimal Push Optimisation

[Sound familiar?](@/optimising-recursive-ascent.md#stack-access-minimisation) In the last post we had a detailed discussion of this optimisation that comes down to: don't push states onto the stack that you don't read the value of. The relevant states that we actually match on are 0, 5, 6, and 7, as before. This removes a lot of pushes and pops from states:

```rust
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
            S10 => {
                let _ = stack.pop(); // 7
                outprod("T = T * F");
                label = TGoto
            },
```

How's our performance now?

```
parse_reverse_goto/a+a*(a+a)*a
                        time:   [138.27 ns 138.47 ns 138.65 ns]

parse_push_first/a+a*(a+a)*a
                        time:   [210.75 ns 210.97 ns 211.18 ns]

parse_minpush/a+a*(a+a)*a
                        time:   [147.08 ns 147.23 ns 147.38 ns]
Found 14 outliers among 100 measurements (14.00%)
  7 (7.00%) low mild
  4 (4.00%) high mild
  3 (3.00%) high severe
```

Hmm, getting close to the original `reverse_goto` time, but not exactly impressive.

Well, last time we got the biggest bump from inlining states that are only referenced in one place. So let's try that again.

## Inline States with Single Reference

You might be surprised to learn that with the two steps above, we've made a _lot_ of states available for inlining. We can in fact inline all states except for `S0`, and the `_Goto` states!

```rust
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
```

What you're looking at here is `TGoto` with states `S9` and `S2` inlined into it. `EGoto` looks similar, with `S8` and `S1` inlined: a match on the top of the stack, a look at the next thing in the input, and similar but not equal match statements. 

Now I might be extrapolating from a overly simple example here, but I think it may be generally good to exchange the match on the next character with the match on the stack number after inlining things into a `_Goto` state. In our case what this does is allow us to avoid checking the stack in case of a `*`, and unify the other case were we set the `label` to `EGoto`. This means we can inline `EGoto`. 

In `FGoto` we can simply note that it always goes to `TGoto` afterwards, so we can just inline `TGoto` after the top-of-stack `match` code (since that's the only place where it's used). This means we've eliminated two more states, and now we only have `S0` and `FGoto` left.

I've kept both the simple inlined version as `inline1` and the second one with just two states as `inline2` to see the performance:

```
parse_reverse_goto/a+a*(a+a)*a
                        time:   [138.27 ns 138.47 ns 138.65 ns]

parse_push_first/a+a*(a+a)*a
                        time:   [210.75 ns 210.97 ns 211.18 ns]

parse_minpush/a+a*(a+a)*a
                        time:   [147.08 ns 147.23 ns 147.38 ns]

parse_inline1/a+a*(a+a)*a
                        time:   [125.38 ns 125.54 ns 125.70 ns]
Found 12 outliers among 100 measurements (12.00%)
  2 (2.00%) low mild
  7 (7.00%) high mild
  3 (3.00%) high severe

parse_inline2/a+a*(a+a)*a
                        time:   [117.35 ns 117.54 ns 117.73 ns]
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) low mild
  5 (5.00%) high mild
  1 (1.00%) high severe

```

Cool, we got there. We reduced the states to _only two_! We got faster than where we started. Everything is great. Just for fun, let's compare to where we ended up last time in terms of performance with a whole 9 states left:

```
parse_max_inline/a+a*(a+a)*a
                        time:   [56.606 ns 56.721 ns 56.840 ns]
```

Well, boo. At first I tried to improve on my code, tested a few more tricks, including duplicating a bunch of code. Things got faster, but I also duplicated a bunch of `outprod` calls. I took another look at the old `parse_max_inline`, and then I found it: a bug in the code! I forgot to push state number 7 onto the stack in `S7` :( 

### The Performance Was a Lie

```
parse_max_inline/a+a*(a+a)*a
                        time:   [97.611 ns 97.769 ns 97.924 ns]
                        change: [+72.289% +72.740% +73.150%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
```

Suddenly our `parse_inline2/a+a*(a+a)*a` result looks a lot less silly. Still not _impressive_, I was expecting to do better here. But hey, at least they're pretty close.

Find [all the code in my blog's repo](https://github.com/Apanatshka/blog/tree/zola/code/optimising-recursive-ascent-part-2).

## Conclusion

I've been playing it fast and loose by doing all these optimisations by hand. Consider how few test and benchmark inputs I've actually used, and the results start to smell pretty fishy :\ Learning nothing from this, I feel pretty confident that my latest implementation is correct and the numbers are good :D [^sarcasm]

Now I need to put an end to this blog post and this topic of optimising recursive ascent parsers if I want to reach the next parsing topic. You see, I'm tempted to dive into a deeper investigation of what makes `push_first` so slow. And I want to know the performance of the techniques from the previous post if we skip chain elimination, which duplicates `outprod` calls. But I also want to tell you about _generalised_ parsers, and I've been reading a bit about _error recovery_ as well. Plenty more cool new things to discover, if I can just let go of this for a little while.

A final note: pushing state numbers onto the stack as numbers is apparently dumb. If you make it an enum, `rustc` can significantly optimise the code, I saw a 25% improvement on my benchmark. Though again, that's on a single input. But then I'm not writing a research paper here, now am I? ¯\\\_(ツ)\_/¯

[^sarcasm]: That was a joke. I thought I should clarify, in case you didn't pick up on the sarcasm ^^
