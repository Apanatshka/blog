+++
title = "LR Parsing and Recursive Ascent"
date = "2024-04-08"
taxonomies.tags = ["theory of computation", "automata", "context-free grammar", "pda", "parsing"]
+++

This is part 2 of old-school linear time parsing algorithms, which only need to go over the input once, without backtracking or caching. In [part 1](@/ll-parsing-recursive-descent.md) we learned about LL parsing, how to construct the parse tables for it, and how those relate to direct execution of the parser with recursive descent. Since part 1 and part 2 were originally one blog post that I simply cut in half after feedback that it was too long, this post assumes you've read part 1. It's probably still pretty readable without reading all of part 1 though. This post is meant to be readable for people unfamiliar with parsing, and yet be interesting for those who are familiar with the more traditional explanations! It's still interesting because I like to explain things from an automata point of view instead of a procedural algorithm. We'll check out LR parsing, its (different) parse tables, and recursive _ascent_. I'm hoping that last one is something you don't know about yet, it's pretty cool! I'll use examples of grammars, and tables, and automata, and even some Rust code to show you how to implement a parser. Let's dive in!

# Bottomup, LR parsing

LR stands for left-to-right, rightmost derivation _in reverse_. If you think about it, left-to-right and rightmost derivation are incompatible: The rightmost derivation chooses the rule for the rightmost sort first every time, but that means skipping over some unknown amount of input if you read left-to-right to even get to that point. However, the _reverse_ of the rightmost derivation is a left-to-right form of parsing. This reverse derivation describes going bottomup, left-to-right through the parse tree.

## Expressive power and relation to LL

One of the biggest upsides of LR(k) parsing is its __expressivity__. The set of all LL(k) languages of any _k_ is a strict subset of all LR(1) languages. Note that this is speaking of languages, not grammars. For grammars it holds that any LL(k) grammar for a specific _k_ is also an LR(k) grammar, and not necessarily the other way around.

An LR(k) grammar of any k greater than 1 can be automatically transformed into an LR(1) grammar that is not necessarily structurally equivalent. This is highlights the difference between grammar and language level equivalence. We can basically capture any LR language in an LR(1) grammar, but LR with larger _k_ may be able to describe the language in a nicer way (smaller grammar).

A good overview of how LL and LR relate to each other on the grammar and language level is [summarised on the Computer Science Stack Exchange](https://cs.stackexchange.com/a/48). In the comments someone suggests making a list of examples for each of these relationships, which seems like a great idea, but not something I have the patience for right now. This blog post has enough scope creep already.

## How LR works

In order to give a reverse rightmost derivation, we need to figure what sorts can be at the leftmost leaf of the parse tree for our LR grammar. Then we try to apply the rules for those sorts all simultaneously. And to do so we can't just use the automaton build we've used for LL.

Remember that the automata we've used previously mapped well on recursive descent, and showed us where to use an LL parse table with look-ahead to resolve ambiguity. Crucially, those automata observe every rule we go into. But for LR we need to explore down all the rules simultaneously. Let's see if we can't get somewhere with that idea and the example grammar of the language that wasn't LL:

| | |
:- | :-
$S = a S$          | (1)
$S = A$            | (2)
$A = a A b$        | (3)
$A = \varepsilon$  | (4)

We start again with the separate automata for each rule:

{{ digraph(gz_file="parsing-and-all-that/lr-rule-automata.gv", alt="Simple automata for each grammar rule from the example") }}

Now in order to explore to the bottom-left of the parse tree, we need to be free to go into any rule. So we will connect the rules again to the nodes that expect a certain sort, but with epsilon transitions so we don't observe how far down we are or with what rule in particular we got there. We'll need that later, but let's not worry about that until we have the downward exploration.

{{ digraph(gz_file="parsing-and-all-that/lr-single-automaton-epsilon.gv", alt="Partially constructed automaton using the automata from the grammar rules, using epsilon transitions") }}

Obviously this is not a full automaton model of a parser yet, but it allows us to always go down to the next leaf of the parse tree without using the stack. Let's clean up the automaton with an NFA-to-DFA conversion:

{{ digraph(gz_file="parsing-and-all-that/lr-single-automaton-dfa.gv", alt="Partially constructed automaton using the automata from the grammar rules, after merging states through NFA-to-DFA conversion") }}

This is almost exactly how an LR(0) automaton would be drawn. Instead of S₁₀ and S₁₁, you write out the "LR item"s `S = . a S` and `S = a . S`. But otherwise it would be pretty much this. This is considered a PDA, though what's happening on the stack is left implicit. That's because what's actually happening on the stack of LR automata is very structured, but a little involved. That makes the PDA harder to read and draw, but I'll demonstrate it once:

{{ digraph(gz_file="parsing-and-all-that/lr-single-automaton-explicit.gv", alt="A fully explicit PDA that does LR parsing") }}

This should look quite familiar. We're pushing inputs on the stack as we consume them, paired with the state we're in at that point. And then we're popping whole bodies of rules off the stack and replacing them with the sort of that rule. The first thing is called a _shift_ action, the second is called a _reduce_ action. We've seen this trick before in the naive PDA built from a CFG, all the way at the start of this post in the refresher. But this time we get an automaton with more states.

Notice that _where_ a reduce action goes depends on originating state of the last thing that's popped. That's why we keep track of those on the stack. When we reduce by rule 3 (state A₃), depending on whether the `a` came from box 1 or box 0, we go to different places. This is easier to see in our proper LR(0) automaton, where box 1 points to state S₁ with a transition labeled `A`. This is a _goto_ action. In an LR parse table interpreter, the _goto_ is a separate action that immediately follows a _reduce_ action, which merely returns to the last popped state. When a reduce just returns that's also more like a function call and return, so that's nice. Anyway, that's also why a reduce transition in the above automaton always labels the originating state of the pushed sort the same as the last thing popped from the stack.

Something worth repeating now that we're looking at the details: LL decides what rule to take _before_ consuming the input for that rule, whereas LR decides what rule to take _after_ consuming all the input for that rule. In other words, we only reduce by a rule when we've seen the entire body of the rule, that's why there's less trouble with look-ahead.

Speaking of look-ahead: we have some shift-reduce problems in our automaton. And by that I mean: how do we choose when to shift and when to reduce when both are an option? This is a determinism issue in our current automaton, and just like in our LL automaton, we solve it with look-ahead (and yes, that can and will later be summarised in a parse table). Our latest automaton gives a clear view of what we will be able to do if we reduce, so the look-ahead follows from what input can be consumed next after each reduce:

{{ digraph(gz_file="parsing-and-all-that/lr-single-automaton-explicit-lookahead.gv", alt="A fully explicit PDA that does LR parsing, with look-ahead") }}

As you can see, we need at most 1 look-ahead to deterministically parse this grammar. We're sometimes looking ahead to the end-of-input represented with `$`. The look-ahead makes this an LALR(1) grammar; what that is and why it's different from normal LR(1) is what we'll see in the next section. 

## LR parsetable construction and expressivity

Let's look at some example grammars, how to construct their tables, and when you need a better parsetable construction method.

### LR(0)

LR(0) does not look ahead but just reduces whenever possible. If there are multiple options, you have a shift-reduce or a reduce-reduce conflict. Shift-shift conflicts don't exist in LR since the NFA-to-DFA process would have merged the two states such conflicting transitions would point to.
Let's look at an LR(0) grammar:

| | |
:- | :-
$S = E 2$          | (1)
$E = E 1$          | (2)
$E = 1$            | (3)

The LR automaton for this grammar is:

{{ digraph(gz_file="parsing-and-all-that/lr-zero.gv", alt="An LR(0) automaton for the above grammar") }}

The corresponding parse table follows this automaton:

<div class="parsetable">

|           | `1`  | `2`  | `$`      | `E`  |
|:----------|:----:|:----:|:--------:|:----:|
| **Box0**  | s E₃ |      | _accept_ | Box1 |
| **Box1**  | s E₂ | s S₁ |          |      |
| **E₃**    | r 3  | r 3  | r 3      |      |
| **E₂**    | r 2  | r 2  | r 2      |      |
| **S₁**    | r 1  | r 1  | r 1      |      |
</div>

The transition from box 0 to E₃ that shifts `1` becomes a shift action to $E_3$ in the row of box 0 and the column of `1`. The transition from box 0 to box 1 with `E` becomes a goto to box 1 in the row of box 0 and column of `E`. Finally a state that's at the end of a rule will get all reduce actions by that rule (indicated by its number) in the column for input. Accepting the input is typically based on look-ahead of the end-of-input. 

### Simple LR (SLR)

The smallest motivating example for Simple LR is the following grammar that parses the same language as before:

| | |
:- | :-
$S = E 2$          | (1)
$E = 1 E$          | (2)
$E = 1$            | (3)

Notice how rule 2 is now right-recursive instead of left-recursive. It's a nice symmetry how left-recursive rules give you trouble in LL, and right-recursive rules _could_ give you trouble in LR[^indirect-recursion]. 

{{ digraph(gz_file="parsing-and-all-that/simple-lr.gv", alt="An LR(0) automaton for the above grammar") }}

<div class="parsetable">

|           | `1`          | `2`  | `$`      | `E`  |
|:----------|:------------:|:----:|:--------:|:----:|
| **Box0**  | s Box1       |      | _accept_ | S₁₁  |
| **Box1**  | s Box1 / r 3 | r 3  | r 3      | E₂   |
| **S₁₁**   |              | s S₁ |          |      |
| **S₁**    | r 1          | r 1  | r 1      |      |
| **E₂**    | r 2          | r 2  | r 2      |      |
</div>

Yay, we have a shift-reduce conflict. How do we solve it? By not blindly putting a reduce in the entire row of a state that could reduce. If we check the _Follow_ set of the sort we're reducing to (we defined that when we built LL parse tables, remember?), we can put the reduce action in only the column of the terminals that are in that follow set. If we look at the grammar, we can see that only `2` can follow `E`. So the SLR table for this grammar is:

<div class="parsetable">

|           | `1`    | `2`  | `$`      | `E`  |
|:----------|:------:|:----:|:--------:|:----:|
| **Box0**  | s Box1 |      | _accept_ | S₁₁  |
| **Box1**  | s Box1 | r 3  |          | E₂   |
| **S₁₁**   |        | s S₁ |          |      |
| **S₁**    |        |      | r 1      |      |
| **E₂**    |        | r 2  |          |      |
</div>

### Look-Ahead LR (LALR)

From now on we'll be looking at reduce-reduce conflicts only. While you can get shift-reduce conflicts with the following algorithms through grammars that don't fit (due to ambiguity or requiring more look-ahead than you're taking into account), when you give an LALR(k) grammar to an SLR(k) algorithm you can only get reduce-reduce conflicts. Same with an LR(k) grammar put through the LALR(k) algorithm.

Here our example grammar that just barely doesn't work with SLR:

| | |
:- | :-
$S = a E c$          | (1)
$S = a F d$          | (2)
$S = b F c$          | (3)
$E = e$              | (4)
$F = e$              | (5)

See how rules 4 and 5 are the same except they have different sort names? Yeah, that's going to be "fun" if they're used with the same prefix like in rules 1 and 2. Let's have a look at the automaton and SLR parse table.

{{ digraph(gz_file="parsing-and-all-that/look-ahead-lr.gv", alt="An LR(0) automaton for the above grammar") }}

<div class="parsetable">

|           | `a`    | `b`    | `c`       | `d`  | `e`    | `$`      | `E`  | `F`  |
|:----------|:------:|:------:|:---------:|:----:|:------:|:--------:|:----:|:----:|
| **Box0**  | s Box1 | s Box2 |           |      |        | _accept_ |      |      |
| **Box1**  |        |        |           |      | s Box3 |          | S₁₂  | S₂₂  |
| **Box2**  |        |        |           |      | s F₅   |          | S₃₂  |      |
| **Box3**  |        |        | r 4 / r 5 | r 5  |        |          |      |      |
| **S₁₂**   |        |        | s S₁      |      |        |          |      |      |
| **S₁**    |        |        |           |      |        | r 1      |      |      |
| **S₂₂**   |        |        |           | s S₂ |        |          |      |      |
| **S₂**    |        |        |           |      |        | r 2      |      |      |
| **S₃₂**   |        |        | s S₃      |      |        |          |      |      |
| **S₃**    |        |        |           |      |        | r 3      |      |      |
| **F₅**    |        |        | r 5       | r 5  |        |          |      |      |
</div>

The reduce-reduce conflict, as promised. It's in box 3, where we can reduce by E₄ or F₅, when the look-ahead is `c`. This is because the look-ahead sets of both `E` and `F` contain `c` due to rules 1 and 3. If we look at the automaton though, we can clearly see that if we reduce and we have a `c` next, we should reduce by `E`.

Look-Ahead LR parsing uses basically this method, analysing what shifts can happen after certain reduces. Putting it is algorithmic terms, LALR doesn't use LL _Follow_ sets, but defines more accurate _Follow_ sets based on the automaton. Each instance of the start of a rule in the automaton (F₅₀ in boxes 1 and 2) gets a separate _Follow_ set computed. That's how we resolve the conflict with LALR:

<div class="parsetable">

|           | `a`    | `b`    | `c` | `d`  | `e`    | `$`      | `E`  | `F`  |
|:----------|:------:|:------:|:---:|:----:|:------:|:--------:|:----:|:----:|
| **Box3**  |        |        | r 4 | r 5  |        |          |      |      |
</div>

Note that since the LALR _Follow_ sets follow directly from the automaton, this is basically the same as the intuition given at the end of the [previous section](#how-lr-works).

### LR(1)

I like this LALR parsing story. It's so intuitive with the NFA-to-DFA conversion, just looking at the automaton to see the follow sets. But, it's doesn't give you the complete power of deterministic push-down automata. I present to you the previous example grammar with one more rule:

| | |
:- | :-
$S = a E c$          | (1)
$S = a F d$          | (2)
$S = b F c$          | (3)
$E = e$              | (4)
$F = e$              | (5)
$S = b E d$          | (6)

This results in an automaton that's almost the same as before:

{{ digraph(gz_file="parsing-and-all-that/lr-one-zero.gv", alt="An LR(0) automaton for the above grammar") }}

We now have a reduce-reduce conflict in box 3 again. With look-ahead `a` you can reduce to both `E` and `F`. Same for look-ahead `b` by the way. It _is_ deterministically decidable which one we should reduce to, but it basically now depends on which state we came from.

With LALR we build an automaton for each rule, and try to reuse that rule independent of the context in which it is used. That's keeps our process simple, our automaton small, but it also causes us to lose exactly the information we need to resolve the reduce-reduce conflict in box 3 above: the left context. I know the information is technically on the stack, but our parsing process decides on the rule to reduce by based on the state and look-ahead only. 

LR(k) automata/parsers keep the same parsing process still, they just have larger automata in which their states summarise the left context. We're basically going to distinguish almost every occurrence of a sort in the grammar, similar to when we made our LL(2) grammar strong:

{{ digraph(gz_file="parsing-and-all-that/lr-one.gv", alt="An LR(1) automaton for the above grammar") }}

How do we do this? We duplicate each rule for each terminal in the LL follow set of the sort of that rule. We annotate each of those rules with that terminal. Now we do our usual thing: rule to automaton, epsilons, NFA-to-DFA. But when wiring the epsilons, extra terminal annotations should now match up with the _LALR_ follow set of the occurrence of the sort.

With this particular example, the automaton looks almost the same. There's a bit more fluff with the annotations, but they basically propagate the look-ahead for each rule. Which means we can distinguish the context in which `E` and `F` are used differently! In general though, duplicating each rule for each terminal in the LL follow set leads to a very large amount of rules, and plenty of the time this isn't necessary... LR(1) automata have lots of redundant states that do basically the same thing and would have been merged in LALR without any reduce-reduce conflicts.

### Parse table construction algorithm

You've already seen parse table construction by automaton for both LL and the many flavours of LR now. And you've seen parse table construction by _First_ and _Follow_ set for LL. Parse table construction for LR will of course also require _First_ and _Follow_ sets, sometimes including more accurate _Follow_ sets for particular occurrences of sorts. It's mostly an iterative build-up of the NFA-to-DFA (powerset construction) though. I'm not going to detail that in this post.

While researching the material, I found some claims for _minimal_ LR(1) algorithms, which create LALR(1) tables when possible, and slightly larger tables when necessary. They look interesting, but quite something to figure out, and I haven't gotten to what I wanted to write about most yet, so that will have to wait until some other time. Perhaps I'll include the normal LR parse table construction algorithm there too as a start.

## Recursive Ascent

We finally get to the original impetus for this blog post: recursive ascent parsing. As you might be able to guess, this is the LR analogue to recursive _descent_ for LL. So we're going to write code that directly executes the LR automaton instead of simulating it by parse table interpretation.

Before, in recursive descent parsing, we saw that rules and sorts become functions. Rules call sort functions to parse a sort, and sorts check the look-ahead to choose a rule by which to parse the alternative of the sort. Basically grammar rules became functions, and the parse table was split into functions for each sort.

In recursive _ascent_ parsing we will turn states of the LR automaton into functions. Each function will shift or reduce based on the input and call the corresponding state for that edge. Let's expand our LR(1) example a little bit, and then take a look at the recursive ascent parsing:

| | |
:- | :-
$S = a E c$          | (1)
$S = a F d$          | (2)
$S = b F c$          | (3)
$E = e e$            | (4)
$F = e e$            | (5)
$S = b E d$          | (6)
$S = b e e a$        | (7)

The reason for the extra `e`s in rules 3 and 4 is to show how that increases the LR(1) automaton size. We'll now have 4 states instead of 2 + an LALR reduce-reduce conflict. The reason for adding rule 7 is so we have a state where we might shift or reduce depending on the look-ahead, which influences the code we generate. Let's check out the automaton first:

{{ digraph(gz_file="parsing-and-all-that/recursive-ascent.gv", alt="An LR(1) automaton for the above grammar") }}

Perhaps making both changes at the same time makes this a bad example to show off LR(1) automaton size... If you imagine the automaton without rule 7 you'll see that boxes 3 and 4 are the same except for their ingoing and outgoing edges. This is what happens with longer rules and having to distinguish the final state of the rules for a different look-ahead (boxes 5 and 6 here).

The other notable difference is that we now have a box 6 that can both shift and reduce. This will make the code for the recursive ascent more interesting. Let's start with the basics:

```rust
use std::env;
use std::iter::Peekable;

type Iter<'a> = Peekable<std::slice::Iter<'a, Terminal>>;

type Terminal = char;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Sort {
    S,
    E,
    F,
}

/// Box0, the starting state of our automaton.
/// Itemset:
/// S = . a E c
/// S = . a F d
/// S = . b F c
/// S = . b E d
/// S = . b e e a
fn box0(input: &mut Iter) -> Result<(), String> {
    match input.next() {
        None => Err(String::from("Unexpected end of input: S = .")),
        Some('a') => match box1(input)? {
            Sort::S => Ok(()),
            s => unreachable!("Unexpected sort: S = a {s:?}"),
        },
        Some('b') => match box2(input)? {
            Sort::S => Ok(()),
            s => unreachable!("Unexpected sort: S = b {s:?}"),
        },
        Some(c) => Err(format!("Unexpected input: S = . {c}")),
    }
}
```

This should look familiar from the recursive descent parser code. The notable difference is that we now have a function name less connected to the grammar, and more to the LR automaton. This makes it harder to understand the code, stack traces, etc. 

```rust
/// Box1
/// Itemset:
/// S = a . E c
/// S = a . F d
/// E = . e
/// F = . e
fn box1(input: &mut Iter) -> Result<Sort, String> {
    match input.next() {
        None => Err(String::from(
            "Unexpected end of input while parsing S = a . E; S = a . F",
        )),
        Some('e') => match box3(input)? {
            Sort::E => s12(input),
            Sort::F => s22(input),
            s => unreachable!("Unexpected sort: S = a {s:?}"),
        },
        Some(c) => Err(format!("Unexpected input: S = a . {c}")),
    }
}

/// S = a E . c
fn s12(input: &mut Iter) -> Result<Sort, String> {
    match input.next() {
        None => Err(String::from(
            "Unexpected end of input while parsing S = a E . c",
        )),
        Some('c') => s1(input),
        Some(c) => Err(format!("Unexpected input: S = a E . {c}")),
    }
}

/// S = a E c .
fn s1(input: &mut Iter) -> Result<Sort, String> {
    match input.next() {
        None => Ok(Sort::S),
        Some(c) => Err(format!("Unexpected input: S = a E c . {c}")),
    }
}

/// S = a F . d
fn s22(input: &mut Iter) -> Result<Sort, String> {
    match input.next() {
        None => Err(String::from(
            "Unexpected end of input while parsing S = a F . d",
        )),
        Some('d') => s2(input),
        Some(c) => Err(format!("Unexpected input: S = a F . {c}")),
    }
}

/// S = a F d .
fn s2(input: &mut Iter) -> Result<Sort, String> {
    match input.next() {
        None => Ok(Sort::S),
        Some(c) => Err(format!("Unexpected input: S = a F d . {c}")),
    }
}

/// Box3
/// Itemset:
/// E = e . e (c)
/// F = e . e (d)
fn box3(input: &mut Iter) -> Result<Sort, String> {
    match input.next() {
        None => Err(String::from(
            "Unexpected end of input while parsing E or F.",
        )),
        Some('e') => box5(input),
        Some(c) => Err(format!("Unexpected input: E = e . {c} ; F = e . {c}")),
    }
}

/// Box5
/// Itemset:
/// E = e e . (c)
/// F = e e . (d)
fn box5(input: &mut Iter) -> Result<Sort, String> {
    match input.peek() {
        None => Err(String::from(
            "Unexpected end of input while parsing E or F.",
        )),
        Some('c') => Ok(Sort::E),
        Some('d') => Ok(Sort::F),
        Some(c) => Err(format!("Unexpected input: E = e e . {c} ; F = e e . {c}")),
    }
}
```

This bit of code should give you an idea of the code pattern in the "easy case". Each state either shifts in one-or-more rules it's in (e.g. `s12`, `box3`), shifts into a new rule expecting a sort back to use for the goto (e.g. `box1`), or reduces (e.g. `s1`, `box5`).

```rust
/// Box2
/// Itemset:
/// S = b . F c
/// S = b . E d
/// S = b . e e a
/// E = . e
/// F = . e
fn box2(input: &mut Iter) -> Result<Sort, String> {
    match input.next() {
        None => Err(String::from(
            "Unexpected end of input while parsing E or F.",
        )),
        Some('e') => match box4(input)? {
            (0, Sort::F) => s32(input),
            (0, Sort::E) => s62(input),
            (1, Sort::S) => Ok(Sort::S),
            s => unreachable!("Unexpected return/sort: S = b {s:?}"),
        },
        Some(c) => Err(format!("Unexpected input: S = b . {c}")),
    }
}
```

This is the point where things start looking different. In box 2 we might shift `e` because we've entered rules 4 or 5 which will reduce to `E` or `F`. But we could also be in rule 7. If the result from box 4 is that we were in rule 7, we need to go back to the previous caller. So function `box4` returns a pair of the number of returns left to go and the sort we're reducing to. This way we can distinguish the two cases and take the appropriate action.

If you want to keep a recursive ascent code generator simpler you can of course always return a pair. You could also generate the code in [_continuation passing style_](https://en.wikipedia.org/wiki/Continuation-passing_style), where you pass a function that takes the sort and does the goto action instead of accepting a pair as a result. But because the Rust compiler is not very good at tail call optimisation, so I'm not doing that pattern here.

```rust
/// S = b F . c
fn s32(input: &mut Iter) -> Result<Sort, String> {
    match input.next() {
        None => Err(String::from(
            "Unexpected end of input while parsing S = b F . c",
        )),
        Some('c') => s3(input),
        Some(c) => Err(format!("Unexpected input: S = b F . {c}")),
    }
}

/// S = b F c .
fn s3(input: &mut Iter) -> Result<Sort, String> {
    match input.next() {
        None => Ok(Sort::S),
        Some(c) => Err(format!("Unexpected input: S = b F c . {c}")),
    }
}

/// S = b E . d
fn s62(input: &mut Iter) -> Result<Sort, String> {
    match input.next() {
        None => Err(String::from(
            "Unexpected end of input while parsing S = b E . d",
        )),
        Some('d') => s6(input),
        Some(c) => Err(format!("Unexpected input: S = b E . {c}")),
    }
}

/// S = b E d .
fn s6(input: &mut Iter) -> Result<Sort, String> {
    match input.next() {
        None => Ok(Sort::S),
        Some(c) => Err(format!("Unexpected input: S = b E d . {c}")),
    }
}

/// Box4
/// Itemset:
/// S = b e . e a
/// E = e . e (d)
/// F = e . e (c)
fn box4(input: &mut Iter) -> Result<(usize, Sort), String> {
    match input.next() {
        None => Err(String::from(
            "Unexpected end of input while parsing E or F.",
        )),
        Some('e') => box6(input).map(decr),
        Some(c) => Err(format!(
            "Unexpected input: S = b e . {c}; E = e . {c} ; F = e . {c}"
        )),
    }
}

/// helper
fn decr((c, s): (usize, Sort)) -> (usize, Sort) {
    (c - 1, s)
}
```

Note how in `box4` we're now calling the decrement helper function after the call to `box6` to count one `return` we're going to do immediately after.

```rust
/// Box6
/// Itemset:
/// S = b e e . a
/// E = e e . (d)
/// F = e e . (c)
fn box6(input: &mut Iter) -> Result<(usize, Sort), String> {
    match input.peek() {
        None => Err(String::from(
            "Unexpected end of input while parsing E or F.",
        )),
        Some('c') => Ok((2, Sort::F)).map(decr),
        Some('d') => Ok((2, Sort::E)).map(decr),
        Some('a') => {
            input.next();
            s7(input).map(decr)
        }
        Some(c) => Err(format!("Unexpected input: E = e . {c} ; F = e . {c}")),
    }
}
```

The number of returns to do is equal to the size of the body of the rule we are reducing. Of course we immediately decrement because we are going to immediately return, hence the `map(decr)`.

```rust
/// S = b e e a .
fn s7(_input: &mut Iter) -> Result<(usize, Sort), String> {
    Ok((4, Sort::S)).map(decr)
}

fn lex(input: String) -> Vec<Terminal> {
    input.chars().collect()
}

pub fn main() -> Result<(), String> {
    let input = env::args().next().expect("Argument string to parse");
    let input = lex(input);
    let mut input = input.iter().peekable();
    box0(&mut input)
}
```

In our main function we can call `box0` with the input. Since this is LR(1) we only need a peekable iterator, that can look ahead 1 terminal.

### Table size = Code size

With both recursive descent and recursive ascent parsing, we're representing the parsing logic directly in code, not as an explicit data representation of a parse table. As such, if you have a larger parse table, you get more code. In LR, when LALR doesn't suffice, parse tables can potentially grow quite large, as we saw to a limited extent with the last example. 

## Recursive Ascent-Descent Parsing

Have you noticed that in the recursive ascent code there are some pretty boring and tiny looking functions? I'm talking about `s12`, `s1`, `s22`, `s2`, `s32`, `s3`, `s62`, `s6`. These will likely be targeted by the inliner of the Rust compiler[^inlining], but aren't they a bit much to even generate?

The common denominator of these functions, and the states of the LR automaton they correspond to, is that they have already honed in on a single rule from the grammar and are only parsing that. Kind of like in an LL parser, except we used the LR automaton mechanism to select the rule instead of an LL look-ahead. If we follow that idea to its logical conclusion, we can do LL parsing from any point where we know there's only one rule left (or equivalently, inline those simple functions). This means we only have box functions left:

```rust
fn box1(input: &mut Iter) -> Result<Sort, String> {
    match input.next() {
        None => Err(String::from(
            "Unexpected end of input while parsing E or F.",
        )),
        Some('e') => match box3(input)? {
            Sort::E => {
                consume(input, 'c')?;
                Ok(Sort::S)
            }
            Sort::F => {
                consume(input, 'd')?;
                Ok(Sort::S)
            }
            s => unreachable!("Unexpected sort: S = a {s:?}"),
        },
        Some(c) => Err(format!("Unexpected input: S = a . {c}")),
    }
}
```

This is using the `consume` function from the recursive descent parser example from before.

```rust
/// Box6
/// Itemset:
/// S = b e e . a
/// E = e e . (d)
/// F = e e . (c)
fn box6(input: &mut Iter) -> Result<(usize, Sort), String> {
    match input.peek() {
        None => Err(String::from(
            "Unexpected end of input while parsing E or F.",
        )),
        Some('c') => Ok((2, Sort::F)).map(decr),
        Some('d') => Ok((2, Sort::E)).map(decr),
        Some('a') => {
            input.next(); // consume 'a'
            Ok((3, Sort::S)).map(decr)
        }
        Some(c) => Err(format!("Unexpected input: E = e . {c} ; F = e . {c}")),
    }
}
```

Note that in box 6 we now count the number of symbols in the body of the rule before the dot to come up with the number of returns.

### Left Corners?

The left corner of a rule in the grammar is the left-most symbol in the body of the rule, plus the left corners of any sorts in left corner. So it's basically a _First_ set with the sorts included. I found this is some of the older literature, and figured I'd add a note for myself in here.

There is/was such a thing as left-corner parsing, seemingly mostly used in natural language processing (NLP). NLP mostly uses _ambiguous_ context-free grammars, and typically uses (used?) a backtracking parser to deal with that. These can be slow of course. And it turns out left corners helped with this, by adding some "filtering" that allows the parser to backtrack less. This is connected to recursive ascent-descent parsing, which you could also see as filtering with LR to finish parsing with LL. In our case we just don't do backtracking.

# Fin

I really need to stop working on this blog post and publish it already. It's been over a year since I started working on it (on and off, during holidays when I had enough focus time)[^graphviz]. I already had an idea of where to go to next (generalised parsers), but now I also want to study minimal LR(1) automaton/parse table algorithms, and look at continuation passing style again because I think you can pass the left-context as a function argument. This would give you an LALR automaton structure with LR parsing power. Is that a good idea? Don't know, needs testing (or reading papers/blog posts, probably someone else already tried this before). In the mean time I've also been learning about some optimisation techniques to apply on recursive ascent code if you generate it, which makes them look really great in terms of code size and hopefully also performance.

I usually have a pithy remark or sneak the Kaomoji into the footnotes, but I must be out of practice, because I can't think of a good way to do that...

Ehh, whatever ¯\\\_(ツ)\_/¯


[^indirect-recursion]: _Indirect_ left recursion is even worse in LL. At least the direct version can still be dealt with by an automatic grammar rewrite algorithm. That's more or less what the node-reparenting trick mentioned at the end of the LL section does. Similarly, there are automatic grammar rewrites for direct right-recursion for LR, and indirect right recursion can be more problematic...

[^inlining]: Actually, I checked in [Compiler Explorer](https://godbolt.org/) how this turns out, and while `s7` is inlined and compiled away entirely, adapting `box1` to consume directly will make the assembly at `opt-level=3` smaller. Adding an `#[inline]` hint on `consume` helps as well. Though I may just be seeing the effect of uniform error messages through `consume`. Actually following and understanding the optimised assembly code is a pain, so I just clicked around a bit to verify that the example code is reduced to a state machine with jumps and labels instead of using function `call` instructions. So that's neat, basically what I was hoping for :)

[^graphviz]: I hope you appreciate how much time it took to find example grammars to steal (or occasionally develop myself) and especially how much time it took to get GraphViz to output somewhat decent automata of those examples!
