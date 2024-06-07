+++
title = "LL Parsing and Recursive Descent"
date = "2024-06-07"
aliases = ["compsci/2024/04/07/parsing-and-all-that"]
taxonomies.tags = ["theory of computation", "automata", "context-free grammar", "pda", "parsing"]
+++

Hello again! I'm picking up my [series on Automata](@/2016-03-28-theory-of-computation.md), with this post that goes into what I had always meant to get to: parsers. We'll check out the old-school linear time parsing algorithms, which only need to go over the input once, without backtracking or caching. Originally this was one big post, but given the feedback I've gotten from friends, I've now split it up into two. In this first post we'll check out LL, parse tables, and recursive descent. This post is meant to be readable for people unfamiliar with parsing, and yet be interesting for those who are familiar with the more traditional explanations but explaining things from an automata point of view instead of by rote algorithm. I'll use examples of grammars, and tables, and automata, and even some Rust code to show you how to implement a parser. The [second post](@/2024-06-09-lr-parsing-recursive-ascent.md) is on LR parsing. Enjoy!

# Refresher from Pushy Automata

We'll start with a brief refresher from the previous post of the series, [pushy automata](@/2016-05-15-pushy-automata.md), since that was a little while back.

## Push-Down Automata

Push-down automata (PDAs) are automata with a _stack_. Normal finite automata just consume input and have fixed memory via their states. PDAs can remember things on a single stack too, by pushing onto it and popping from it. Here's a deterministic PDA for recognising the language of words that start with zeroes, followed by an equal number of ones:

{{ digraph(gz_file="pushy-automata/non-regular-deterministic.gv", alt="Non-regular language example, deterministic") }}

So we start at $q_0$, see if there is a $0$ as input, ignore the top of the stack, and put a $\$$ on the stack as a marker for the end of the stack. Now we're in state $q_1$, in which we can consume more zeroes from the input and put those on the stack. If we find a one as input, we remove a zero from the stack by not pushing anything new on the stack. Now we're in state $q_2$ where we remove zeroes from the stack for every one in the input, until we consume the final one by removing the $\$$ from the stack.

> Aside: This is one of the examples from the old blog post, and I now see that it is missing a transition! This automaton rejects the input $01$, because there is no transition $q_1 \to q_3$ labeled $1,\$ \to \varepsilon$. Oops ^_^

## Context-Free Grammars, Derivations, and a naive PDA translation

A context-free grammar that describes the above language is:

| | |
:- | :-
$S = 0 S 1$        | (step)
$S = \varepsilon$  | ($\varepsilon$)

Sort $S$ is the start symbol, the starting point in the grammar. If we're using the grammar _productively_ we start from the start symbol and use the rules left-to-right to replace sorts until we get the sentence in the corresponding context-free language that we want. Something like: $ S \to 0 S 1 \to 0 0 S 1 1 \to 0 0 1 1 $. This is called a _derivation_.

The most general, naive way to derive a push-down automaton for any context-free grammar is by pushing the end-of-stack and start symbol at the start, having a "main" state that uses the grammar rules with the body reversed (to deal with the stack order), and an accept state that pops the end-of-stack:

{{ digraph(gz_file="parsing-and-all-that/binary-grammar.gv", alt="Naive PDA of the above binary grammar") }}

Here the stack grows left-to-right, so the lowest symbol on the stack is $ (end of stack), followed by S (the grammar start symbol). By the rules of the grammar we can manipulate the top of the stack and rewrite it to the body. If the input lines up with what we have on the stack, we can eliminate both. It's simple, but inefficient because of all the nondeterminism.

## Derivations, Parse Trees and Ambiguity

Let's look at a slightly more interesting grammar from a parser perspective:

| | |
:- | :-
$S = S + S$  | (add)
$S = S * S$  | (mul)
$S = 1$      | ($\varepsilon$)

When you want to derive $1 + 1 * 1$ , you can do this in all manner of ways. The following derivation picks just an arbitrary sort on which to apply a rule from the grammar:

1. $S$  (first S)
2. $S + S$  (first S)
3. $1 + S$  (first S)
4. $1 + S * S$  (second S)
5. $1 + S * 1$  (first S)
6. $1 + 1 * 1$

Notice how in some steps the leftmost $S$ was replaced, while in others the rightmost was replaced. Generally speaking, you'll want either a leftmost or a rightmost derivation for parsers, which is to say: a grammar rule is always applied to the leftmost or rightmost sort. There are three reasons for this. The first is that you want a parser to be predictable in when it applies grammar rules, as you may connect so-called _semantic actions_ to each rule. These are pieces of code that are run when the parser applies the particular rule. (A typical example is a simple calculator). Such actions could perform side-effects, therefore order matters. For this reason, leftmost vs rightmost can also be observed. Two other reasons you to want this predictable derivation order is ease of implementation, and ease of proving things about your algorithm. These last two care less for whether it's leftmost or rightmost.

The most common semantic actions I'm aware of is to build a syntax tree with a parser. This builds a tree structure out of the parsed text. A parse tree, or concrete syntax tree, contains all the rule applications as seen in the grammar. An abstract syntax tree abstracts over some parts of the syntax tree, such as leaving out whitespace, or parentheticals (the shape of the tree captures the precedence anyway), or injections (grammars rules of the form $S_1 = S_2$ ). Let's look at some parse trees of the last example, $1 + 1 * 1$ :

{{ digraph(gz_file="parsing-and-all-that/parse-trees.gv", alt="Parse trees of 1 + (1 * 1) and (1 + 1) * 1") }}

Notice how the leaves of the two trees are in the same order left-to-right as the input, but for the left tree the plus is higher up in the tree while in the right tree the star is higher up. If we want to interpreter the input as simple arithmetic, where multiplication binds tighter than addition, the left tree is the one we want. This is the predecedence of the operators, $* > +$ .

When you can get multiple trees like this, the grammar is called ambiguous. More formally, if you use only leftmost derivations (or only rightmost) and still find two distinct derivations that give the same sentence, the grammar is ambiguous. So to be clear: the above trees can be created with only leftmost derivations, it's not a matter of choosing one or the other for the two trees. Derivation order (leftmost or rightmost) has to do with _side-effect order_ of semantic actions only. When you build trees you don't really need side-effects, so the derivation order has no effect on it.

**With that recap out of the way:** For the purposes of _this_ blog post, we'll look at <em>un</em>ambiguous grammars for the languages we want to parse. Still, whether you use leftmost derivation or rightmost derivation in a parser that parses unambiguous grammars matters quite a lot in terms of what languages you can describe deterministically. It also influences how easily you can write a parser by hand for such a grammar, and how easily you can (programmatically) explain why your parser _doesn't_ accept certain inputs (parser error messages). So let's have a look at LL and LR parsing techniques, where the first L in those abbreviations stands for Left-to-right (as in reading direction in text), and the second letters are respectively leftmost and rightmost derivation.

# Topdown, (Strong) LL parsing

To take a good look at LL parsing, we will first work with a grammar that is not ambiguous or left-recursive:

| | |
:- | :-
$S = F$          | (1)
$S = ( S + F )$  | (2)
$F = a$          | (3)

So sort $S$  is the start symbol, we also have sort $F$ , and we have round brackets, plusses, and $a$ 's. This is enough information to create a table that, based on (1) the next sort to be parsed and (2) the next symbol in the input, predicts which rule from the grammar to use to parse the input further. In other words, if you know where you are in the input and grammar, you can look ahead at the next symbol of input and tell which *unique* grammar rule predicts the next bit of input (assuming the input fits the grammar). The table for the above grammar looks like so:

<div class="parsetable">

|                        | `(` | `a` |
| :--------------------- | --: | --: |
| $S$ |   2 |   1 |
| $F$ |     |   3 |
</div>

A table like the above is an LL(1) parse table, because it uses only 1 symbol of "look-ahead" in the columns. LL(1) grammars are always strong LL grammars, which means that they only need the combination of the sort to be parsed and the next symbol(s) to decide on a unique grammar rule to apply. In general, LL(k) grammars do not have to be strong, and if they are not, you also need to know what was already parsed from the input (the "left context") in order to choose a unique grammar rule[^LLdef]. For example, the following grammar is LL(2), and not strong:

| | |
:- | :-
$S = A\ a\ b\ A\ b\ a$  | (1)
$A = a$                 | (2)
$A =$                   | (3)

You can see this if you try to write an LL(2) parse table for it:

<div class="parsetable">

|                        | `a a` | `a b` | `b a` |
| :--------------------- | ----: | ----: | ----: |
| $S$ |     1 |     1 |       |
| $A$ |     2 |   2,3 |     3 |
</div>

If you look ahead to `a b` on the input, and the next sort is $A$, then it really depends on whether you are at the start of the input or in the middle of rule 1. If you're at the start, you must choose rule 3 so you can parse `a b` as part of the rule 1, but if you're already in the middle of rule 1, you must choose rule 2 for $A$ so you can continue to parse `b a` of rule 1.

If you mark $A$ in rule 1 with where you are in rule 1 ($ S = A₁\ a\ b\ A₂\ b\ a $), you get an LL(2) grammar that is strong, although the table for it is larger[^table]:

<div class="parsetable">

|                          | `a a` | `a b` | `b a` |
| :----------------------- | ----: | ----: | ----: |
| $S$   |     1 |     1 |       |
| $A_1$ |     2 |     3 |       |
| $A_2$ |       |     2 |     3 |
</div>

In general, you can always use this trick to construct a strong, _structurally equivalent_ LL grammar with the same look-ahead. This is quite useful for constructing simple LL parsers. However, the downside of these parsers is that on wrong input they can fail later than a more complicated LL(k) parser that works for the non-strong grammar. And that matters if you want to give nice error messages.

### An intuition for table construction by automaton

Building the above tables was a matter of keeping in mind what they mean, and squinting a little. But in the case of a larger grammar, or a parsetable generator, of course you want an exact process. Before I dive into _First_ and _Follow_ sets that are the traditional method for building these tables, let me give you a method that is less practical but in my opinion more intuitive.

Step 1: Let's build a simple automaton for each rule of the grammar, where we assume both sorts and terminals are on the input. 

{{ digraph(gz_file="parsing-and-all-that/ll-rule-automata.gv", alt="Simple automata for each grammar rule from the last example") }}

Note how each node of a rule automaton has the number of the rule followed by the offset into the body of the rule. The state represents where we are in the rule while parsing by that rule. The last node doesn't have this offset so you can easily identify it, even when it's no longer a final state.

Typically you'll find texts on parsers display the position in a rule more visually with "LR item" notation. This uses the actual rule and a dot to indicate where we are in the rule. While this makes individual automata states more readable, it makes the automata large and therefore harder to display in a readable way as a whole. That's why you won't find that notation in this post's automata. But just to illustrate an example of the notation:

<div class="parsetable">

| Shorthand in this blog   | LR Item notation                                   |
| :----------------------- | :------------------------------------------------- |
| S₁₀                      | $S = .\ A\ a\ b\ A\ b\ a$ |
| S₁₁                      | $S = A\ .\ a\ b\ A\ b\ a$ |
| S₁₅                      | $S = A\ a\ b\ A\ b\ .\ a$ |
| S₁                       | $S = A\ a\ b\ A\ b\ a\ .$ |
</div>

Step 2: Now instead of consuming a sort with an automaton, we'll use $\varepsilon$ rules to jump to the automata of the rules for that sort instead. We'll use the PDA stack with unique labels to get back to where you would be after consuming the sort.

{{ digraph(gz_file="parsing-and-all-that/ll-single-automaton.gv", alt="Single PDA using the automata from the grammar rules") }}

The $\downarrow{}X$ is an abbreviation for an $\varepsilon, \varepsilon \rightarrow X$ edge that pushes a symbol on the stack unconditionally, it was hard to get graphviz to cooperate on node placement of this graph otherwise... Now at every node that had a sort transition you have multiple transition options, these are where you need to look ahead. Soooo...

Step 3: at a sort transition node, for each $\downarrow$ transition, follow transitions until you've consumed _k_ terminals (2 in this example) from the input. These are the terminals of the column in the parse table, and the rule of the $\downarrow$ transition gets put into that cell. You can also put the look-ahead into the automaton:

{{ digraph(gz_file="parsing-and-all-that/ll-single-automaton-lookahead.gv", alt="Single PDA using the automata from the grammar rules with lookahead noted") }}

### Building LL tables for strong LL grammars by traditional method

While the above building of automata gives a visual intuition, it's not the most efficient way to express how we can build LL tables. The traditional method does the same thing in essence, but using some pre-computed sets to calculate the cells in the table.

A cell at the row labeled with sort $A$ and the column labeled with terminal(s) $v$ should have the grammar rule $A = w$ (where $w$ is a mix of terminals and sorts or $\varepsilon$), under the following condition: $v$ is in the _First_ set of $w$, or $\varepsilon$ is in the _First_ set of $w$ and $v$ is in the _Follow_ set of $A$. In other words: $v \in \textit{First}(w) \cdot \textit{Follow}(A)$

Let's unpack that. The _First_ set of a terminal is a singleton set with that terminal. The _First_ set of a sort is the set of first non-terminals that the sort can expand to, directly or indirectly. So a rule $A = a [...]$ causes $a$ to appear in the _First_ set of $A$, $A = B [...]$ causes the _First_ set of $B$ to be included in the _First_ set of $A$, and $A = \varepsilon$ causes $\varepsilon$ to appear in the _First_ set of $A$. This last rule says $A$ can be expanded to "nothing", so if that's an option we need to check the _Follow_ set of $A$.

The _Follow_ set is basically every non-terminal that can follow $A$ in the grammar. So when you have $B = [...] A\ a [...]$, $a$ is in the _Follow_ set of $A$. A rule $B = [...] A$ causes the _Follow_ set of $B$ to be included in the _Follow_ set of $A$. And the _Follow_ set of the start symbol has the end-of-file 'terminal' $\$$.

The _Follow_ set is basically every non-terminal that can follow $A$ in the grammar. So when you have $B = [...] A\ a [...]$, $a$ is in the _Follow_ set of $A$. A rule $B = [...] A$ causes the _Follow_ set of $B$ to be included in the _Follow_ set of $A$. And the _Follow_ set of the start symbol has the end-of-file 'terminal' $\$$.

Finally, there is the dot operator between the _First_ and _Follow_ sets: this is a _truncated product_, that takes every combination of the two sets, sticks them together (in order), and truncates to length k. That's a bit of an abstraction over the k in LL(k), which I didn't take into account in the explanation of _First_ and _Follow_ sets. The _First_ sets should have length k strings of course, and so you may need to take more _First/Follow_ sets into account when computing these. Another thing I glossed over is that we actually use the _First_ set of $w$, a mix of terminals and sorts on the right-hand side of our grammar rules. If $w$ is $a\ A\ B\ b$, then its _First_ set is $\{a\} \cdot \textit{First}(A) \cdot \textit{First}(B) \cdot \{b\}$.

Ok, with that all done, we can use those tables. But before we do, a quick word about expressive power, because LL is not particularly powerful...

### Limitations and Expressive power

There are always languages that cannot be captured by an LL(k) grammar that can be captured by an LL(k+1) grammar. In other words, look-ahead size is important in the expressivity of an LL grammar, and LL(k) for any specific k does not capture _all_ deterministic context-free languages[^nondet].

We've already seen an example of an LL(2) grammar, but what would be a language that cannot be captured by any LL(k)? Take the language of a's followed by b's, where the number of a's $\geq$ the number of b's. Or as a grammar:

| | |
:- | :-
$S = a S$          | (1)
$S = A$            | (2)
$A = a A b$        | (3)
$A = \varepsilon$  | (4)

The problem for LL here is that we would have to look ahead in the input until we read the entire input before we could decide whether we can start consuming the input with Rule 1 or Rule 2 (and then Rule 3). 

There is a class of grammars called LL-regular (LLR) grammars captures all LL(k) grammars for any k and slightly more. These LLR grammars are cool in that they are still parseable in linear time, as long as you have something called a "regular partition" of your grammar. Getting that is an undecidable problem though. And since there is an LR(1) grammar that is not in LLR, this stuff is the trifecta of confusing, impractical, and less powerful[^LLR] than a much more useful technique that we will cover later in this post: LR. But first, going from tables to parsers!

## Predictive Parsing

Since we already know what the tables mean, we can write a simple parse table interpreter to finish our _predictive parser_. The parser is called predictive because based on the _k_ look-ahead terminals, we decide the grammar rule to use to continue parsing, which typically predicts some of the structure of the input well beyond the part we peeked at for the look-ahead.

Ok, let's write a quick parse table interpreter for our LL(2) example. We'll start with some definitions.

```rust
use std::collections::HashMap;
use std::env;

use lazy_static::lazy_static;
use peekmore::PeekMore;

type Terminal = char;

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

#[derive(Debug, Eq, PartialEq)]
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
        stack.push(Symbol::Terminal('a'));
        stack.push(Symbol::Terminal('b'));
        stack.push(Symbol::Sort(Sort::A2));
        stack.push(Symbol::Terminal('b'));
        stack.push(Symbol::Terminal('a'));
        stack.push(Symbol::Sort(Sort::A1));
    }

    fn aa(stack: &mut Vec<Symbol>) {
        stack.push(Symbol::Terminal('a'));
    }

    #[allow(clippy::ptr_arg)]
    fn a_epsilon(_stack: &mut Vec<Symbol>) {}
}
```

Clippy is great for catching all kinds of poor code, but for consistency I've chosen to `#[allow]` this time. Note that to effectively run a context-free grammar on a PDA, you need to push the symbols in your rules on the stack in reverse, [as mentioned in the recap](#context-free-grammars-derivations-and-a-naive-pda-translation).

```rust
lazy_static! {
    static ref TABLE: HashMap<(Sort, Terminal, Terminal), Rule> = {
        let mut table = HashMap::new();
        assert_eq!(None, table.insert((Sort::S,  'a', 'a'), Rule::S));
        assert_eq!(None, table.insert((Sort::S,  'a', 'b'), Rule::S));
        assert_eq!(None, table.insert((Sort::A1, 'a', 'a'), Rule::Aa));
        assert_eq!(None, table.insert((Sort::A1, 'a', 'b'), Rule::AEpsilon));
        assert_eq!(None, table.insert((Sort::A2, 'a', 'b'), Rule::Aa));
        assert_eq!(None, table.insert((Sort::A2, 'b', 'a'), Rule::AEpsilon));
        table
    };
}
```

Nothing very special really, just encoding what we had already. The main parse loop is also very unexciting now that we have implemented most of the logic of the grammar already. We basically manage the stack, eliminating terminals on the stack with those from the input and applying rules from the table based on sort and look-ahead, and give errors if we get unexpected input:

```rust
pub fn lex(input: String) -> Vec<Terminal> {
    input.chars().collect()
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
                if let Some(&&actual) = input.next() {
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

type Terminal = char;

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
        &[Some('a'), Some('a')] => s(input),
        &[Some('a'), Some('b')] => s(input),
        &[term1, term2] => Err(format!("Unexpected {term1:?} {term2:?} while parsing S")),
        _ => Err("Unexpected end of file.".to_owned()),
    }
}

fn sort_A1(input: &mut Iter) -> Result<(), String> {
    // A1
    match input.peek_amount(2) {
        &[Some('a'), Some('a')] => a_a(input),
        &[Some('a'), Some('b')] => a_epsilon(input),
        &[term1, term2] => Err(format!("Unexpected {term1:?} {term2:?} while parsing A")),
        _ => Err("Unexpected end of file.".to_owned()),
    }
}

fn sort_A2(input: &mut Iter) -> Result<(), String> {
    // A2
    match input.peek_amount(2) {
        &[Some('a'), Some('b')] => a_a(input),
        &[Some('b'), Some('a')] => a_epsilon(input),
        &[term1, term2] => Err(format!("Unexpected {term1:?} {term2:?} while parsing A")),
        _ => Err("Unexpected end of file.".to_owned()),
    }
}

fn s(input: &mut Iter) -> Result<(), String> {
    sort_A1(input)?;
    consume(input, 'a')?;
    consume(input, 'b')?;
    sort_A2(input)?;
    consume(input, 'b')?;
    consume(input, 'a')
}

fn a_a(input: &mut Iter) -> Result<(), String> {
    consume(input, 'a')
}

fn a_epsilon(_input: &mut Iter) -> Result<(), String> {
    Ok(())
}
```

Our parse table has now become code directly, with these functions named after the sorts of the rows. They call rules that are also functions, which in turn use the sort functions. Those rules also use `consume`, this time without having to reverse the order of the rule body.

```rust
pub fn lex(input: String) -> Vec<Terminal> {
    input.chars().collect()
}

pub fn main() -> Result<(), String> {
    let input = env::args().next().expect("Argument string to parse");
    let input = lex(input);
    let mut input = input.iter().peekmore();
    sort_s(&mut input)
}
```

Finally, our main function just calls the right sort function instead of putting that sort on the stack. And the loop is gone, since we now use recursion.

## Summary of LL, and an insight from the automaton

We've now seen LL(k) parsing, left-to-right leftmost derivation. This leftmost derivation directly corresponds to walking through the parse tree topdown, depth-first, leftmost child first. Whenever we expand a leftmost sort by a rule for that sort, we have to choose a rule, therefore we use the look-ahead (with a length of _k_) to see ahead and choose based on this.

We've seen an LL(1) and an LL(2) grammar, and in general more look-ahead allows us to parse more grammars _and_ more languages. Both are important: certain languages cannot be expressed in LL(1) or LL(2), and some LL(1) grammars are harder to read and write than the LL(2) grammar of the same language.

We've seen how we can construct simple DFAs for each rule in our grammar, and then replace the sort transitions $N_1 \rightarrow^{A} N_2$ by a (PDA) push transition ($\downarrow A$) from $N_1$ to all starts of DFAs corresponding to rules of $A$, and a pop transition ($\uparrow A$) from the ends of those DFAs to $N_2$. Then the LL table, the decision table of sort + look-ahead = rule, naturally follows from this PDA by finding what input will be consumed if a certain rule is chosen, and using that as the look-ahead to make the decision for that rule.

The recursive descent way of writing a parser directly as code is nice and simple, it really just follows the grammar. Since you're writing plain old code with function calls, you can imagine people have found nice ways to extend and adapt the pattern of recursive descent parsers. For one, it's quite easy to reason about where you are in the parse when hitting an error state, which makes it fairly easy to give friendly error messages when the parser doesn't accept an input. You can also use a trick to fix up direct left-recursion called [node reparenting](https://en.wikipedia.org/wiki/Tail_recursive_parser), where you use a loop or tail-recursion locally construct the tree bottom-up. You could argue that such a parser is a hybrid between recursive descent and ascent, a "recursive descent-ascent parser".

Finally, if we look back at the automaton, we can see that the PDAs we build have a very strict shape. We either have a non-deterministic choice due to multiple push transitions for a sort, or we have predicted input, a single path of terminals to consume from the input. If we think back to the [NFAs and DFAs](@/2016-04-10-finite-automata.md) from early on in this blog post series, those used the input to chose what state to go to next. Now we have single-path DFAs that just consume input, and a separate table on a look-ahead to resolve non-determinism from the pushes and pops. The strict shape here indicated that we're not really making full use of the power of automata. This will change with the next parsing technique.

# Continue?

Now that you've learned all about LL parsing, would you like to learn about LR parsing? About how it's more powerful, also uses parse tables with some of the same construction tricks, also has nice corresponding push-down automata, and even has an analogue to recursive descent? Then click on to [part 2](@/2024-06-09-lr-parsing-recursive-ascent.md)! Or bookmark it for later, when you are able to absorb information again or whatever ¯\\\_(ツ)\_/¯


[^LLdef]: I'm fairly sure my prose description there is the same as a formal definition, and it feel a bit nicer to think about than [the ones you can find on Wikipedia](https://en.wikipedia.org/wiki/LL_grammar#Formal_definition).

[^table]: Technically you'd need to see $A_1$  and $A_2$  as separate symbols and duplicate the rules for $A$ , resulting in a larger grammar in correspondence with the larger table. But I couldn't be bothered, and the parse table as shown works just as well. This is relevant to the code size of a recursive descent parser too, since you can just reuse the code for rules 2 and 3 instead of having duplicate code for the two extra rules. What's a recursive descent parser? That comes just a little later in the post, so keep reading ;)

[^nondet]: Yes, there are non-deterministic context-free languages. Those are the context-free languages that can only be parsed with a non-deterministic PDA. Since this post is about deterministic parsers, we'll ignore the non-deterministic languages.

[^LLR]: While I find the [Wikipedia article on LLR](https://en.wikipedia.org/wiki/LL_grammar#Regular_case) confusing, and it makes a good case for why it's not really used, I'm still somewhat intrigued. This is one of those things that will stay on my reading list for a while I think, something I still want to understand further...
