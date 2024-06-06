+++
title = "Parsing and all that"
date = "2024-04-07"
tags = ["theory of computation", "automata", "pda", "parsing"]
+++

Hello again! I'm picking up my [series on Automata](@/2016-03-28-theory-of-computation.md), with this post that goes into what I had always meant to get to: parsers. We'll check out the old-school linear time parsing algorithms, which only need to go over the input once, without backtracking or caching. We'll check out LL and LR, parse tables, recursive descent and recursive _ascent_. Welcome to the world of deterministic parsing...

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

Something worth repeating now that we're looking at the details: LL decides what rule to take _before_ consuming the input for that rule, whereas LR decides what rule to take _after_ consuming all the input for that rule. In other words, we only reduce by a rule when we've seen the entire body of the rule, that why there's less trouble with look-ahead.

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

I really need to stop working on this blog post and publish it already. It's been over a year since I started working on it (on and off, during holidays when I had enough focus time)[^graphviz]. I already had an idea of where to go to next (generalised parsers), but now I also want to study minimal LR(1) automaton/parse table algorithms, and look at continuation passing style again because I think you can pass the left-context as a function argument. This would give you an LALR automaton structure with LR parsing power. Is that a good idea? Don't know, needs testing (or reading papers/blog posts, probably someone else already tried this before).

I usually have a pithy remark or sneak the Kaomoji into the footnotes, but I must be out of practice, because I can't think of a good way to do that...

Ehh, whatever ¯\\\_(ツ)\_/¯

# Footnotes

[^LLdef]: I'm fairly sure my prose description there is the same as a formal definition, and it feel a bit nicer to think about than [the ones you can find on Wikipedia](https://en.wikipedia.org/wiki/LL_grammar#Formal_definition).

[^table]: Technically you'd need to see $A_1$  and $A_2$  as separate symbols and duplicate the rules for $A$ , resulting in a larger grammar in correspondence with the larger table. But I couldn't be bothered, and the parse table as shown works just as well. This is relevant to the code size of a recursive descent parser too, since you can just reuse the code for rules 2 and 3 instead of having duplicate code for the two extra rules. What's a recursive descent parser? That comes just a little later in the post, so keep reading ;)

[^nondet]: Yes, there are non-deterministic context-free languages. Those are the context-free languages that can only be parsed with a non-deterministic PDA. Since this post is about deterministic parsers, we'll ignore the non-deterministic languages.

[^LLR]: While I find the [Wikipedia article on LLR](https://en.wikipedia.org/wiki/LL_grammar#Regular_case) confusing, and it makes a good case for why it's not really used, I'm still somewhat intrigued. This is one of those things that will stay on my reading list for a while I think, something I still want to understand further...

[^indirect-recursion]: _Indirect_ left recursion is even worse in LL. At least the direct version can still be dealt with by an automatic grammar rewrite algorithm. That's more or less what the node-reparenting trick mentioned at the end of the LL section does. Similarly, there are automatic grammar rewrites for direct right-recursion for LR, and indirect right recursion can be more problematic...

[^inlining]: Actually, I checked in [Compiler Explorer](https://godbolt.org/) how this turns out, and while `s7` is inlined and compiled away entirely, adapting `box1` to consume directly will make the assembly at `opt-level=3` smaller. Adding an `#[inline]` hint on `consume` helps as well. Though I may just be seeing the effect of uniform error messages through `consume`. Actually following and understanding the optimised assembly code is a pain, so I just clicked around a bit to verify that the example code is reduced to a state machine with jumps and labels instead of using function `call` instructions. So that's neat, basically what I was hoping for :)

[^graphviz]: I hope you appreciate how much time it took to find example grammars to steal (or occasionally develop myself) and especially how much time it took to get GraphViz to output somewhat decent automata of those examples!
