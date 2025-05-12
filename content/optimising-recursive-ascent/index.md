+++
title = "Optimising Recursive Ascent Parsing"
date = "2024-07-14"
updated = "2024-08-10"
taxonomies.tags = ["theory of computation", "automata", "nfa", "dfa", "rust"]
+++

This is post picks up where we [left off with parsing](@/lr-parsing-recursive-ascent.md): Recursive Ascent. In the previous post I highlighted how parsing is all about grammars and (push-down) automata (PDA). And that if you follow the logic of how LL parsing has recursive descent, then LR parsing should have recursive _ascent_. Which it does!

In this post we'll explore a couple more techniques for making the recursive ascent parser a bit smaller and faster. We'll explore the ideas from [a 1990 paper called _Optimizing Directly Executable LR Parsers_ by Peter Pfahler](https://doi.org/10.1007/3-540-53669-8_82). For the example grammar, the paper can optimise away 6 out of 15 states in the parser!

## (LA)LR and Recursive Ascent

Let me quickly recap the main things we'll need from the previous post, while introducing you to the example grammar we'll be working with through this post. The grammar is a simple arithmetic grammar that has been made unambiguous by encoding the precedent relation between multiplication and addition (multiplication binds tighter):

| | |
:- | :-
$S = E$     | (1)
$E = E + T$ | (2)
$E = T$     | (3)
$T = T * F$ | (4)
$T = F$     | (5)
$F = a$     | (6)
$F = ( E )$ | (7)

I'm going to cut the construction steps and go straight to the LALR automaton from this:

{{ digraph(gz_file="single-automaton-dfa.gv", alt="Partially constructed automaton using the automata from the grammar rules, after merging states through NFA-to-DFA conversion") }}

The parse table representation of this DFA is the following:

<div class="parsetable">

|            | `a`    | `+`    | `*`    | `(`    | `)`     | `$`      | | `E`  | `T`  | `F`   |
|:-----------|:------:|:------:|:------:|:------:|:-------:|:--------:|-|:----:|:----:|:-----:|
| **Box0**   | s Box4 |        |        | s Box5 |         | _accept_ | | Box1 | Box2 | Box3  |
| **Box1**   |        | s Box6 |        |        |         | r 1      | |      |      |       |
| **Box2**   |        | r 3    | s Box7 |        | r 3     | r 3      | |      |      |       |
| **Box3**   |        | r 5    | r 5    |        | r 5     | r 5      | |      |      |       |
| **Box4**   |        | r 6    | r 6    |        | r 6     | r 6      | |      |      |       |
| **Box5**   | s Box4 |        |        | s Box5 |         |          | | Box8 | Box2 | Box3  |
| **Box6**   | s Box4 |        |        | s Box5 |         |          | |      | Box9 | Box3  |
| **Box7**   | s Box4 |        |        | s Box5 |         |          | |      |      | Box10 |
| **Box8**   |        | s Box6 |        |        | s Box11 |          | |      |      |       |
| **Box9**   |        | r 2    | s Box7 |        | r 2     | r 2      | |      |      |       |
| **Box10**  |        | r 4    | r 4    |        | r 4     | r 4      | |      |      |       |
| **Box11**  |        | r 7    | r 7    |        | r 7     | r 7      | |      |      |       |
</div>

In the previous post we used a recursive ascent code generation pattern where each row in this table becomes a function, and we return sorts along with a number of returns to do (sized to the body of the rule). The _stack_ of the LR parser is the function stack. But if we're going to do optimisations, I think it will be easier to understand if we use an explicit stack. Let's start with some definitions we'll use throughout:

```rust
pub type Iter<'a> = Peekable<Chars<'a>>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Sort { /*   S,*/    E,    T,    F }

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum State_ {    S0,    S1,    S2,    S3,    S4,    S5,    S6,    S7,    S8,    S9,    S10,    S11,    S0Goto(Sort),    S5Goto(Sort),    S6Goto(Sort),    S7Goto(Sort) }

#[inline(never)]
fn outprod(rule: &str) {
    // a semantic action
    eprintln!("{}", rule)
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Error {
    EOF,
    Unexpected(char),
}
```

So we have our input type `Iter` of characters we can peek into without consuming for that single lookahead. We've got `Sort`s, a `State_` enum with a suspicious underscore tacked on. Note how the states are just `S#`, but the ones that have goto actions also have a `S#Goto` version. This was slightly nicer with the function-encoded recursive ascent parser, which had a natural "return" part. On the other hand, this one should be nicer in the popping multiple things from the stack department. To do a "semantic action" when reducing we mark these places with an uninlinable `outprod`. This will just be a placeholder for some expensive operation.There's an `Error` type, no surprises here. 

Let's continue to our first go at some recursive ascent code for the parse table. You don't have to read through and study all of it, have a quick look, then I'll highlight some insights after:

```rust
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
```

We use a mutable variable `label` to hold the current `State_` of the parser. The `stack` is used to keep track of the state numbers in which we pushed a terminal or did the goto on a sort. So a shift action pushes the current state number and sets `label` to the next `State_`. A reduce action pops off all but one state number on the `stack` and uses the final one to decide the `label` with the form `S#Goto`. This goto `State_` handles the shared logic of jumping to the next state based on the sort that was produced.

## Reverse Goto

Feeling unsatisfied with the code duplication of computing the `label` during the reduce? Me too. Since we're not using functions anymore, popping things of the stack is easy but returning to the state that can do the goto is harder now. Thankfully Pfahler has an insight here, that I will summarise as follows: the goto part of the table is already separate, why not flip the script there and handle things per sort instead of per state? This is called the "reverse goto" strategy, where we have a `State` label for each sort, and we branch on the state number. So our state becomes:

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum State {    S0,    S1,    S2,    S3,    S4,    S5,    S6,    S7,    S8,    S9,    S10,    S11,    EGoto,    TGoto,    FGoto }
```

Now we get to have the same start of the parsing function:

```rust
fn parse_reverse_goto(input: &mut Iter) -> Result<(), Error> {
    use State::*;

    let mut stack = vec![];
    let mut label = S0;
    loop {
        match label {
```

Most states remain the same, but a reduce now looks like this:

```rust
            S3 => {
                outprod("T = F");
                label = TGoto
            },
```

How simple, we reduce to a `Sort::T` so the label must be `TGoto`. We pop off all but one state from the body, zero in this case. What does `TGoto` looks like? Well:

```rust
            TGoto => match stack.last().unwrap() {
                6 => label = S9,
                _ => label = S2,
            },
```

The nice thing about our example grammar is that its goto actions are going to one state in a specific origin state, and to another in all other cases, so all `_Goto` states are this short. According to Pfahler this is quite common in practice, having a _default_ case that is.

Apart from shorter code, this reverse goto saves us a real comparison action: We branch on the sort once (on the `State` label) and on the state number once (within the sort goto state). Our earlier approach branched on the state number twice: once on number from the stack to set the `State_` label, once when branching on that label. Cool!

## Chain Elimination

Chain rules are grammar rules with a single right-hand side item. Examples in our grammar are 1, 3, 5, and 6. If you have a state that reduces only a chain rule, you can merge it into the originating state statically, because you know it's the previous one that the reduce goes back to. In our automaton that's states `S3` and `S4`. By inlining those states, you not only don't have to transition to those states, but the sort that's produced can also be used directly for deciding which state to go to next because we know the originating state statically. So of example, state `S0`:

```rust
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
```

On seeing an `a` we normally go to `S4`, reduce a `T`, then go to `S2`. Instead after inlining `S4` and specialising the `TGoto` to the position we're in, we get:

```rust
            S0 => match input.next() {
                Some('a') => {
                    stack.push(0);
                    outprod("F = a");
                    label = S3;
                }
                Some('(') => {
                    stack.push(0);
                    label = S5;
                }
                Some(c) => return Err(Error::Unexpected(c)),
                None => return Err(Error::EOF),
            },
```

Note that `S3` is another state that we're eliminating, so we _chain_ together these eliminations of states:

```rust
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
```

As you can see, we are eliminating jumps that can be predicted to end up in a specific place statically. The only downside is that we're duplicating the `outprod` calls, which are a stand-in for the semantic actions that make the parser useful. If those are a significant amount of code, that means a lot of code duplication. However, chain rules that are _injections_, i.e. rules of the style `A = B` where `A` and `B` are sorts, are typically not observed with semantic actions because they're mostly there to encode something like precedence. 

## Stack Access Minimisation

Normally we push a state number onto the stack whenever we shift. We also keep a state number on the stack if we end up back there after reducing a sort, we're basically "shifting" the sort and using it to _goto_ the next state. Those are the _relevant_ states, the ones that have goto's. The other states that lead toward another reducing state are there mostly for bookkeeping, we don't read their numbers on the stack. If we don't push their state numbers on the stack, then we don't have to pop them from the stack when reducing. That's the big idea, minimal stack accesses. Or as it's called in [a paper that Pfahler cites](https://doi.org/10.1002/spe.4380200602), minimal push optimisation[^min-push].

In our example the relevant states are 0, 5, 6, and 7. If you take a good look at those states, you'll see that 0 is the starting state that marks the start (end?) of the stack. Every time we push the state number of state 5 is when we shift a `(`. This makes sense, we need to account for the number of those to match them with closing `)`. Every time we push state number 6, is when we shift a `+`. Every time we push state number 7, we shift a `*`. These operators have different priority, `*` binds tighter. So, at least in my mind, it makes sense that we need to keep track of those on the stack. Everything else is fluff, things we can eliminate with this optimisation.

If we no longer push every state, we can't just pop to the size of the right-hand side of a grammar rule when reducing. Instead, we'll have to calculate the pop-count based on how we could have gotten to the reducing state. That means that for each reducing state, and each rule in reduction position in that state, we need to find all the paths in the LR automaton that lead to the state and shift the right-hand side of the rule. If those paths have an equal number of relevant states, great, we have a pop-count. If those paths _don't_ have an equal number of relevant states, you can inspect the stack to see if a sometimes-on-the-stack number is part of the path, and conditionally pop that number too. According to Pfahler these conflicts in path length are pretty rare, so the penalty of the conditional pop is negligible. The original paper that introduced minimal push optimisation duplicated states to resolve the ambiguity.

In our example we don't have a conditional pop, but we go from 8 different state numbers to push onto the stack (0, 1, 2, 5, 6, 7, 8, and 9), to only 4. States 9, 10, and 11 all go down to one unconditional pop off the stack instead of two. 

### Improved minpush

Now this next bit is a general improvement by Pfahler on _minpush_ using the fact that we have a reverse goto treatment. It won't gives us anything extra in the running example though, so it'll be a little dry. If you like, you can skip this subsection. 

Key idea: Sometimes you can avoid pushing a relevant state too! One that falls into the group that always gets matched in the _default_ case of a `_Goto` part of the code. But you can only avoid it if the state below it on the stack would also fall within that _default_ case. Here the details matter, you can pick different sets of states to optimise away...

> #### Reduction Splitting
>
> Since during this improvement on minpush we really care about _default_ cases, we can get more of those if we do _reduction splitting_. Give each reduction in the LR automaton a separate number. We'll get more columns in the _goto_ part of our table, but they may be more sparsely populated. The ones that are exact duplicates aren't worth it of course (and yes, for the running example that's what happens). But any others may give us more default cases.

Getting back to the problem of picking the set of states to optimise away: we can make a _push graph_. Each _relevant_ state is a node in the push graph, and there is an undirected edge between two nodes if the states have a different goto entry for the same (reduction split numbered) sort. This basically means one of the two states connected by an edge must be pushed in order to distinguish the situation when the sort associated with that edge is reduced. The well-known _vertex cover problem_ solves the minimal number of nodes in the graph so at least one node is selected of each edge. This is an NP-hard problem though. And we want to optimise for more that just this minimum. Because if we optimise away the states that for most goto columns of the table would fall within the default case, that's fine, we weren't reading those values on the stack anyway. But if we optimise away the ones we would normally branch on if we didn't do stack access minimisation, then those would need to fall into the default case and the usual default case would have to be expanded to a full inspection of those values.

So to solve all this, we build the push graph, then we pick all the nodes whose numbers we'd usually branch on in the `_Goto` states as nodes we'll keep. Then we expand this set of nodes by picking a node connected to the largest number of uncovered edges until we get a vertex cover (this is a typical heuristic approach to solver the optimisation problem non-optimally).

## Conclusion?

This is the end of the recommendations from Pfahler's paper. If we take our code after these three optimisations, and try to inline more states that are only jumped to once, we can inline states 1, 8 and 11, and we can inline `FGoto`. So we only have 9 of the original 15 state labels left in use, due to reverse goto we do less comparisons, and due to minpush we only push 4 of the 8 potential LR state numbers and can avoid a bunch of related pops off the stack too. Pretty good right?

With a few quick-and-dirty criterion benchmarks, I was able to confirm that the changes we made actually make a difference:

```
parse/a+a*(a+a)*a       time:   [128.58 ns 128.80 ns 129.03 ns]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

parse_reverse_goto/a+a*(a+a)*a
                        time:   [133.78 ns 133.95 ns 134.13 ns]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe

parse_chain_elim/a+a*(a+a)*a
                        time:   [101.61 ns 101.82 ns 102.06 ns]
Found 16 outliers among 100 measurements (16.00%)
  4 (4.00%) high mild
  12 (12.00%) high severe

parse_minpush/a+a*(a+a)*a
                        time:   [96.174 ns 96.275 ns 96.387 ns]
Found 15 outliers among 100 measurements (15.00%)
  2 (2.00%) low mild
  8 (8.00%) high mild
  5 (5.00%) high severe

parse_max_inline/a+a*(a+a)*a
                        time:   [56.606 ns 56.721 ns 56.840 ns]
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe
```

You can find the [code for all of this](https://github.com/Apanatshka/blog/tree/zola/code/optimising-recursive-ascent) in the repo of my blog.

Now my original plan for this post was to continue to with some ideas of my own. If we mix and match the ideas from the paper with recursive ascent-descent and a [code sharing idea from the paper Pfahler cites](https://doi.org/10.1002/spe.4380200602), resulting in the removal of 13 out of 15 states! But last time I wrote a really long blog post about parsing, people complained about the length. So this time I'll just leave you with this cliffhanger ¯\\\_(ツ)\_/¯

[^min-push]: Although the optimisation is introduced by a paper Pfahler cites, he puts his own spin on it that builds on top of the reverse goto, so we'll be looking Pfahler's version of the optimisation here.

<hr/>

# Errata

I made a small mistake in writing `parse_max_inline`, with large consequences. I forgot to push state number 7 in state `S7`... This completely borked the benchmark results, inlining didn't do much for performance:

```
parse/a+a*(a+a)*a       time:   [129.85 ns 130.08 ns 130.35 ns]
                        change: [+0.4813% +0.7739% +1.0607%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 17 outliers among 100 measurements (17.00%)
  1 (1.00%) low severe
  5 (5.00%) low mild
  7 (7.00%) high mild
  4 (4.00%) high severe

parse_reverse_goto/a+a*(a+a)*a
                        time:   [132.41 ns 132.60 ns 132.78 ns]
                        change: [-1.2289% -0.9920% -0.7525%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

parse_chain_elim/a+a*(a+a)*a
                        time:   [100.62 ns 100.74 ns 100.86 ns]
                        change: [-1.5909% -1.2311% -0.9017%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 12 outliers among 100 measurements (12.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  6 (6.00%) high severe

parse_minpush/a+a*(a+a)*a
                        time:   [96.682 ns 96.861 ns 97.060 ns]
                        change: [+0.6625% +0.9165% +1.1943%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) high mild
  3 (3.00%) high severe

parse_max_inline/a+a*(a+a)*a
                        time:   [97.611 ns 97.769 ns 97.924 ns]
                        change: [+72.289% +72.740% +73.150%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
```
