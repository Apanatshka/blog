---
layout:   post
title:    "Parsing and all that"
date:     2023-05-21
category: CompSci
tags:     [theory, automata, computation, push-down automata, stack, context-free languages, context-free grammar, context-free]
---

Hello again! I'm picking up my [series on Automata]({% post_url 2016-03-28-theory-of-computation %}), with this post that goes into what I had always meant to get to: parsers. We'll start with a brief refresher from the previous post of the series, [pushy automata]({% post_url 2016-05-15-pushy-automata %}), since that was a little while ago.

# Push-down Automata

Push-down automata (PDAs) are automata with a _stack_. They don't just consume input and have fixed memory in their states, they can remember things on that single stack too, by pushing onto it and popping from it. Here's a deterministic PDA for recognising the language of words that start with zeroes, followed by an equal number of ones:

{% digraph Non-regular language example, deterministic %}
bgcolor="transparent";
rankdir=LR;
node [shape=circle, fixedsize=shape, width=0.5];
start [shape=none, label="", width=0];
q₀ [shape=doublecircle, width=0.4];
q₃ [shape=doublecircle, width=0.4];
start -> q₀;
q₀ -> q₁ [label="0, ε → $"];
q₁ -> q₂ [label="1, 0 → ε"];
q₂ -> q₃ [label="1, $ → ε"];
q₁ -> q₁ [label="0, ε → 0"];
q₂ -> q₂ [label="1, 0 → ε"];
{% enddigraph %}

So we start at {%latex%}q_0{%endlatex%}, see if there is a {%latex%}0{%endlatex%} as input, ignore the stack, and push a {%latex%}\${%endlatex%} on the stack as a marker for the end of the stack. Now we're in state {%latex%}q_1{%endlatex%}, in which we can consume more zeroes from the input and put those on the stack. If we find a one as input, we remove a zero from the stack by not pushing anything new on the stack. Now we're in state {%latex%}q_2{%endlatex%} where we remove zeroes from the stack for every one in the input, until we consume the final one by removing the {%latex%}\${%endlatex%} from the stack.

> Aside: This is one of the examples from the old blog post, and I now see that it is missing a transition! This automaton rejects the input {%latex%}01{%endlatex%}, because there is no transition {%latex%}q_1 \xrightarrow{1,\ \$\ \to\ \varepsilon} q_3{%endlatex%}. Oops ^_^

# Context-Free Grammars, Derivations, Parse Trees

A context-free grammar that describes the above language is:

:- | :-
{%latex%} S = 0 S 1 {%endlatex%}       | {%latex%} \text{(step)} {%endlatex%}
{%latex%} S = \varepsilon {%endlatex%} | {%latex%} \text{(}\varepsilon\text{)} {%endlatex%}

Sort {%latex%}S{%endlatex%} is the start symbol, the starting point in the grammar. If we're using the grammar _productively_ we start from the start symbol and use the rules left-to-right to replace sorts until we get the sentence in the language that we want. Something like: {%latex%} S \to 0 S 1 \to 0 0 S 1 1 \to 0 0 1 1 {%endlatex%}. This is called a _derivation_.

Let's look at a slightly more interesting grammar from a parser perspective:

:- | :-
{%latex%} S = S + S {%endlatex%}       | {%latex%} \text{(add)} {%endlatex%}
{%latex%} S = S * S {%endlatex%}       | {%latex%} \text{(mul)} {%endlatex%}
{%latex%} S = 1 {%endlatex%} | {%latex%} \text{(}\varepsilon\text{)} {%endlatex%}

When you want to derive {%latex%} 1 + 1 * 1 {%endlatex%}, you can do this in all manner of ways. The following derivation picks just an arbitrary sort on which to apply a rule from the grammar:

{%latex%} S \to S + S \to 1 + S \to 1 + S * S \to 1 + S * 1 \to 1 + 1 * 1 {%endlatex%}.

Notice how in some steps the leftmost {%latex%}S{%endlatex%} was replaced, while in others the rightmost was replaced. Generally speaking, you'll want either a leftmost or a rightmost derivation for parsers, which is to say: a grammar rule is always applied to the leftmost or rightmost sort. There are three reasons for this. The first is that you want a parser to be predictable in when it applies grammar rules, as you may connect so-called _semantic actions_ to each rule. These are pieces of code that are run when the parser applies the particular rule. (A typical example is a simple calculator). Such actions could perform side-effects, therefore order matters. For this reason, leftmost vs rightmost can also be observed. Two other reasons you to want this predictable derivation order is ease of implementation, and ease of proving things about your algorithm. These last two care less for whether it's leftmost or rightmost.

The most common semantic actions I'm aware of is to build a syntax tree with a parser. This builds a tree structure out of the parsed text. A parse tree, or concrete syntax tree, contains all the rule applications as seen in the grammar. An abstract syntax tree abstracts over some parts of the syntax tree, such as leaving out whitespace, or parentheticals (the shape of the tree captures the precedence anyway), or injections (grammars rules of the form {%latex%} S₁ = S₂ {%endlatex%}). Let's look at some parse trees of the last example, {%latex%} 1 + 1 * 1 {%endlatex%}:

{% digraph Parse trees of 1 + (1 * 1) and (1 + 1) * 1 %}
bgcolor="transparent";
rankdir=TB;
node [shape=circle, fixedsize=shape, width=0.5, label=S];
subgraph {
Lfirst1 [label=1];
Lplus [label="+"];
Lhidden [shape=none, label="", width=0.5];
Ladd;
Lsecond1 [label=1];
Lstar [label="*"];
Lthird1 [label=1];
Lmul;
Ladd -> Lfirst1;
Ladd -> Lplus;
Ladd -> Lhidden [style="invis"];
Ladd -> Lmul;
Lmul -> Lsecond1;
Lmul -> Lstar;
Lmul -> Lthird1;
}
subgraph {
Rfirst1 [label=1];
Rplus [label="+"];
Rsecond1 [label=1];
Radd;
Rhidden [shape=none, label="", width=0.5];
Rstar [label="*"];
Rthird1 [label=1];
Rmul;
Rmul -> Radd;
Rmul -> Rhidden [style="invis"];
Rmul -> Rstar;
Rmul -> Rthird1;
Radd -> Rfirst1;
Radd -> Rplus;
Radd -> Rsecond1;
}
{% enddigraph %}

Notice how the leaves of the two trees are in the same order left-to-right as the input, but for the left tree the plus is higher up in the tree while in the right tree the star is higher up. If we want to interpreter the input as simple arithmetic, where multiplication binds tighter than addition, the left tree is the one we want. This is the predecedence of the operators, {%latex%} * > + {%endlatex%}.

When you can get multiple trees like this, the grammar is called ambiguous. More formally, if you use only leftmost derivations (or only rightmost) and still find two distinct derivations that give the same sentence, the grammar is ambiguous. So to be clear: the above trees can be created with only leftmost derivations, it's not a matter of choosing one or the other for the two trees. Derivation order (leftmost or rightmost) has to do with _side-effect order_ of semantic actions only. When you build trees you don't really need side-effects, so the derivation order has no effect on it.

Still, whether you use leftmost derivation or rightmost derivation in a parser that parses unambiguous grammars matter quite a lot in terms of what languages you can describe. It also influences how easily you can write a parser by hand for such a grammar, and how easily you can (programmatically) explain why your parser _doesn't_ accept certain inputs. So let's have a look at LL and LR parsing techniques, where the first L in those abbreviations stands of Left-to-right (as in reading direction in text), and the second letters are from leftmost derivative and rightmost derivative.

# Topdown, (Strong) LL parsing

To take a good look at LL parsing, we will first work with a grammar that is not ambiguous or left-recursive:

:- | :-
{%latex%} S = F {%endlatex%}         | {%latex%} \text{(1)} {%endlatex%}
{%latex%} S = ( S + F ) {%endlatex%} | {%latex%} \text{(2)} {%endlatex%}
{%latex%} F = a {%endlatex%}         | {%latex%} \text{(3)} {%endlatex%}

So sort {%latex%} S {%endlatex%} is the start symbol, we also have sort {%latex%} F {%endlatex%}, and we have round brackets, plusses, and {%latex%} a {%endlatex%}'s. This is enough information to create a table that, based on (1) the next sort to be parsed and (2) the next symbol in the input, predicts which rule from the grammar to use to parse the input further. In other words, if you know where you are in the input and grammar, you can look ahead at the next symbol of input and tell which *unique* grammar rule predicts the next bit of input (assuming the input fits the grammar). The table for the above grammar looks like so:

|                        | `(` | `a` |
| :--------------------- | --: | --: |
| {%latex%}S{%endlatex%} |   2 |   1 |
| {%latex%}F{%endlatex%} |     |   3 |

A table like the above is an LL(1) parse table, because it uses only 1 symbol of "lookahead" in the columns. LL(1) grammars are always strong LL grammars, which means that they only need the combination of the sort to be parsed and the next symbol(s) to decide on a unique grammar rule to apply. In general, LL(k) grammars do not have to be strong, and if they are not, you also need to know what was already parsed from the input in order to choose a unique grammar rule[^LLdef]. For example, the following grammar is LL(2), and not strong:

:- | :-
{%latex%} S = A\ a\ b\ A\ b\ a {%endlatex%} | {%latex%} \text{(1)} {%endlatex%}
{%latex%} A = a {%endlatex%}                | {%latex%} \text{(2)} {%endlatex%}
{%latex%} A = {%endlatex%}                  | {%latex%} \text{(3)} {%endlatex%}

You can see this if you try to write an LL(2) parse table for it:

|                        | `a a` | `a b` | `b a` |
| :--------------------- | ----: | ----: | ----: |
| {%latex%}S{%endlatex%} |     1 |     1 |       |
| {%latex%}A{%endlatex%} |     2 |   2,3 |     3 |

If you look ahead to `a b` on the input, and the next sort is {%latex%}A{%endlatex%}, then it really depends on whether you are at the start of the input or in the middle of rule 1. If you're at the start, you must choose rule 3 so you can parse `a b` as part of the rule 1, but if you're already in the middle of rule 1, you must choose rule 2 for {%latex%}A{%endlatex%} so you can continue to parse `b a` of rule 1.

If you mark {%latex%}A{%endlatex%} in rule 1 with where you are in rule 1 ({%latex%} S = A₁\ a\ b\ A₂\ b\ a {%endlatex%}), you get an LL(2) grammar that is strong, although the table for it is larger[^table]:

|                          | `a a` | `a b` | `b a` |
| :----------------------- | ----: | ----: | ----: |
| {%latex%}S{%endlatex%}   |     1 |     1 |       |
| {%latex%}A_1{%endlatex%} |     2 |     3 |       |
| {%latex%}A_2{%endlatex%} |       |     2 |     3 |

In general, you can always use this trick to construct a strong, *structurally equivalent* LL grammar with the same look-ahead. This is quite useful for constructing simple LL parsers. However, the downside of these parsers is that on wrong input they can fail later than a more complicated LL(k) parser that works for the non-strong grammar.

### Building LL tables for strong LL grammars

Building the above tables was a matter of keeping in mind what they mean, and squinting a little. But in the case of a larger grammar, or a parsetable generator, of course you want an exact process.

So a cell in the table at the row labeled with sort {%latex%}A{%endlatex%} and the column labeled with terminal(s) {%latex%}v{%endlatex%} should have the grammar rule {%latex%}A = w{%endlatex%} (where {%latex%}w{%endlatex%} is a mix of terminals and sorts or {%latex%}\varepsilon{%endlatex%}), under the following condition: {%latex%}v{%endlatex%} is in the _First_ set of {%latex%}w{%endlatex%}, or {%latex%}\varepsilon{%endlatex%} is in the _First_ set of {%latex%}w{%endlatex%} and {%latex%}v{%endlatex%} is in the _Follow_ set of {%latex%}A{%endlatex%}. In other words: {%latex%}v \in \textit{First}(w) \cdot \textit{Follow}(A){%endlatex%}

Huh? Well, the _First_ set of a sort is the set of first non-terminals that the sort can expand to, directly or indirectly. So a rule {%latex%}A = a \textrm{\footnotesize[...]}{%endlatex%} causes {%latex%}a{%endlatex%} to appear in the _First_ set of {%latex%}A{%endlatex%}, {%latex%}A = B \textrm{\footnotesize[...]}{%endlatex%} causes the _First_ set of {%latex%}B{%endlatex%} to also be in the _First_ set of {%latex%}A{%endlatex%}, and {%latex%}A = \varepsilon{%endlatex%} causes {%latex%}\varepsilon{%endlatex%} to appear in the _First_ set of {%latex%}A{%endlatex%}. This last rule says {%latex%}A{%endlatex%} can be expanded to "nothing", so if that's an option we need to check the _Follow_ set of {%latex%}A{%endlatex%}.

The _Follow_ set is basically every non-terminal that can follow {%latex%}A{%endlatex%} in the grammar. So when you have {%latex%}B = \textrm{\footnotesize[...]} A\ a \textrm{\footnotesize[...]}{%endlatex%}, {%latex%}a{%endlatex%} is in the follow set of {%latex%}A{%endlatex%}. A rule {%latex%}B = \textrm{\footnotesize[...]} A{%endlatex%} causes the _Follow_ set of {%latex%}B{%endlatex%} to be in the _Follow_ set of {%latex%}A{%endlatex%}. And the _Follow_ set of the start symbol has the end-of-file meta-terminal of course.

Finally, there is the dot operator between the _First_ and _Follow_ sets: this is a truncated product, that takes every combination of the two sets, sticks them together (in order), and truncates to length k. That's a bit of an abstraction over the k in LL(k), which I didn't take into account in the explanation of _First_ and _Follow_ sets. The _First_ sets should have length k strings of course, and so you may need to take more _First/Follow_ sets into account when computing these. Another thing I glossed over is that we actually use the _First_ set of {%latex%}w{%endlatex%}, a mix of terminals and sorts on the right-hand side of our grammar rules. If {%latex%}w{%endlatex%} is {%latex%}v\ A\ B\ x{%endlatex%}, then its _First_ set is {%latex%}\{v\} \cdot \textit{First}(A) \cdot \textit{First}(B) \cdot \{x\}{%endlatex%}.

Ok, with that all done, we can use those tables. But first we need to talk expressive power, because LL is not particularly powerful...

### Expressive power

There are always languages that cannot be captured by an LL(k) grammar that can be captured by an LL(k+1) grammar. In other words, look-ahead size is important in the expressivity of an LL grammar, and LL(k) for any specific k does not capture _all_ context-free languages.

In fact, a class of grammars called LL-regular (LLR) grammars captures all LL(k) grammars for any k and slightly more. These LLR grammars are cool in that they are still parseable in linear time, as long as you have something called a "regular partition" of your grammar. Getting that is an undecidable problem though. And since there is an LR(1) grammar that is not in LLR, this stuff is the trifecta of confusing, impractical, and less powerful[^LLR] than a much more useful technique that we will cover later in this post: LR. But first, going from tables to parsers!

## Predictive Parsing

Since we already know what the tables mean, we can write a simple parse table interpreter to finish our _predictive parser_. The parser is called predictive because based on the _k_ lookahead character, we decide the grammar rule to use to continue parsing, which typically predicts some of the structure of the input well beyond the part we peeked at for the lookahead.

Ok, let's write a quick parse table interpreter for our LL(2) example. We'll start with some definitions.

```rust
use std::collections::HashMap;
use std::env;

use lazy_static::lazy_static;
use peekmore::PeekMore;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Terminal {
    A,
    B,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Sort {
    S,
    A1,
    A2,
}

enum Symbol {
    Sort(Sort),
    Terminal(Terminal),
}

enum Rule {
    S,
    Aa,
    AEpsilon,
}
```

The imports become useful in a second, for now we have our terminals, sorts, a combination type `Symbol`, and the names of our grammar rules. Assuming we keep around a proper PDA stack of symbols, we can write our grammar rules now:

```rust
impl Rule {
    fn apply(&self, stack: &mut Vec<Symbol>) {
        match self {
            Rule::S => Self::s(stack),
            Rule::Aa => Self::aa(stack),
            Rule::AEpsilon => Self::a_epsilon(stack),
        }
    }

    fn s(stack: &mut Vec<Symbol>) {
        stack.push(Symbol::Terminal(Terminal::A));
        stack.push(Symbol::Terminal(Terminal::B));
        stack.push(Symbol::Sort(Sort::A2));
        stack.push(Symbol::Terminal(Terminal::B));
        stack.push(Symbol::Terminal(Terminal::A));
        stack.push(Symbol::Sort(Sort::A1));
    }

    fn aa(stack: &mut Vec<Symbol>) {
        stack.push(Symbol::Terminal(Terminal::A));
    }

    #[allow(clippy::ptr_arg)]
    fn a_epsilon(_stack: &mut Vec<Symbol>) {}
}
```

Clippy is great for catching all kinds of poor code, but for consistency I've chosen to `#[allow]` this time. Note that to effectively run a context-free grammar on a PDA, you need to push the symbols in your rules on the stack in reverse, as mentioned in passing in [the last blog post]({% post_url 2016-05-15-pushy-automata %}#context-free-grammars).

```rust
lazy_static! {
    static ref TABLE: HashMap<(Sort, Terminal, Terminal), Rule> = {
        let mut table = HashMap::new();
        assert_eq!(None, table.insert((Sort::S,  Terminal::A, Terminal::A), Rule::S));
        assert_eq!(None, table.insert((Sort::S,  Terminal::A, Terminal::B), Rule::S));
        assert_eq!(None, table.insert((Sort::A1, Terminal::A, Terminal::A), Rule::Aa));
        assert_eq!(None, table.insert((Sort::A1, Terminal::A, Terminal::B), Rule::AEpsilon));
        assert_eq!(None, table.insert((Sort::A2, Terminal::A, Terminal::B), Rule::Aa));
        assert_eq!(None, table.insert((Sort::A2, Terminal::B, Terminal::A), Rule::AEpsilon));
        table
    };
}
```

Nothing very special really, just encoding what we had already. The main parse loop is also very unexciting now that we have implemented most of the logic of the grammar already. We basically manage the stack, eliminating terminals on the stack with those from the input and applying rules from the table based on sort and lookahead, and give errors if we get unexpected input:

```rust
pub fn lex(_input: String) -> Vec<Terminal> {
    // Out of scope :D
    Vec::new()
}

pub fn main() -> Result<(), String> {
    let input = env::args().next().expect("Argument string to parse");
    let input = lex(input);
    let mut input = input.iter().peekmore();
    let mut stack = Vec::new();
    stack.push(Symbol::Sort(Sort::S));
    while let Some(symbol) = stack.pop() {
        return match symbol {
            Symbol::Terminal(predicted) => {
                if let Some(&&actual) = input.peek() {
                    if predicted == actual {
                        continue;
                    }
                    Err(format!(
                        "Expected terminal {predicted:?}, but got {actual:?}."
                    ))
                } else {
                    Err(format!("Expected terminal {predicted:?}, but got EOF."))
                }
            }
            Symbol::Sort(sort) => {
                if let &[Some(&term1), Some(&term2)] = input.peek_amount(2) {
                    if let Some(r) = TABLE.get(&(sort, term1, term2)) {
                        r.apply(&mut stack);
                        continue;
                    } else {
                        Err(format!(
                            "Unexpected {term1:?} {term2:?} while parsing {sort:?}"
                        ))
                    }
                } else {
                    Err("Unexpected end of input.".to_owned())
                }
            }
        };
    }
    Ok(())
}
```

## Recursive Descent

By encoding the parse table in data, we get some amount of _interpretive overhead_. We have a parse table interpreter with a stack we manage ourselves, but the stack is not really used any different from a call stack. So what if we use function calls instead? That's the idea of _recursive descent_ parsing. It actually makes our code smaller and more straight-forward, which is why it's so popular as a technique for hand-written parsers.

```rust
use std::env;

use peekmore::PeekMore;
use peekmore::PeekMoreIterator;

type Iter<'a> = PeekMoreIterator<std::slice::Iter<'a, Terminal>>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum Terminal {
    A,
    B,
}

fn consume(input: &mut Iter, predicted: Terminal) -> Result<(), String> {
    if let Some(&actual) = input.next() {
        if actual == predicted {
            Ok(())
        } else {
            Err(format!(
                "Expected terminal {predicted:?}, but got {actual:?}."
            ))
        }
    } else {
        Err("Unexpected end of file.".to_owned())
    }
}
```

This time we only need terminals as a type, the rest is gone, and so is the hashmap import for the parsetable. We will need the input, and be able to remove predicted terminals from it, so `consume` comes in handy.

```rust
fn sort_s(input: &mut Iter) -> Result<(), String> {
    // S
    match input.peek_amount(2) {
        &[Some(Terminal::A), Some(Terminal::A)] => s(input),
        &[Some(Terminal::A), Some(Terminal::B)] => s(input),
        &[term1, term2] => Err(format!("Unexpected {term1:?} {term2:?} while parsing S")),
        _ => unreachable!(),
    }
}

fn sort_A1(input: &mut Iter) -> Result<(), String> {
    // A1
    match input.peek_amount(2) {
        &[Some(Terminal::A), Some(Terminal::A)] => a_a(input),
        &[Some(Terminal::A), Some(Terminal::B)] => a_epsilon(input),
        &[term1, term2] => Err(format!("Unexpected {term1:?} {term2:?} while parsing A")),
        _ => unreachable!(),
    }
}

fn sort_A2(input: &mut Iter) -> Result<(), String> {
    // A2
    match input.peek_amount(2) {
        &[Some(Terminal::A), Some(Terminal::B)] => a_a(input),
        &[Some(Terminal::B), Some(Terminal::A)] => a_epsilon(input),
        &[term1, term2] => Err(format!("Unexpected {term1:?} {term2:?} while parsing A")),
        _ => unreachable!(),
    }
}
```

Our parse table has now become code directly, with these functions named after the sorts of the rows.

```rust
fn s(input: &mut Iter) -> Result<(), String> {
    sort_A1(input)?;
    consume(input, Terminal::A)?;
    consume(input, Terminal::B)?;
    sort_A2(input)?;
    consume(input, Terminal::B)?;
    consume(input, Terminal::A)
}

fn a_a(input: &mut Iter) -> Result<(), String> {
    consume(input, Terminal::A)
}

fn a_epsilon(_input: &mut Iter) -> Result<(), String> {
    Ok(())
}
```

Our rules are also functions using `consume` and the sort functions, this time without having to revert any orders.

```rust
pub fn lex(_input: String) -> Vec<Terminal> {
    // Out of scope :D
    Vec::new()
}

pub fn main() -> Result<(), String> {
    let input = env::args().next().expect("Argument string to parse");
    let input = lex(input);
    let mut input = input.iter().peekmore();
    sort_s(&mut input)
}
```

Finally, our main function just calls the right sort function instead of putting that sort on the stack. And the loop is gone, since we now use recursion.

# Bottomup, LR parsing

### LR(0)

### Simple LR

### Lookahead LR

### LR(1)

## Recursive Ascent

# Fin

I usually have a pithy remark or sneak the Kaomoji into the footnotes, but I must be out of practice, because I can't think of a good way to do that...

Ehh, whatever ¯\\\_(ツ)\_/¯

# Footnotes

[^LLdef]: I'm fairly sure my prose description there is the same as a formal definition, and it feel a bit nicer to think about than the ones you can find on [Wikipedia](https://en.wikipedia.org/wiki/LL_grammar#Formal_definition).

[^table]: Technically you'd need to see {%latex%} A₁ {%endlatex%} and {%latex%} A₂ {%endlatex%} as separate symbols and duplicate the rules for {%latex%} A {%endlatex%}, resulting in a larger grammar in correspondence with the larger table. But I couldn't be bothered, and the parse table as shown works just as well. This is relevant to the code size of a recursive descent parser too, since you can just reuse the code for rules 2 and 3 instead of having duplicate code for the two extra rules. What's a recursive descent parser? That comes just a little later in the post, so keep reading ;)

[^LLR]: While I find the [Wikipedia article on LLR](https://en.wikipedia.org/wiki/LL_grammar#Regular_case) confusing, and it makes a good case for why it's not really used, I'm still somewhat intrigued. This is one of those things that will stay on my reading list for a while I think, something I still want to understand further...
