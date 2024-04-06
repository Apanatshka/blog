---
layout:   post
title:    "Parsing and all that"
date:     2024-04-06
category: CompSci
tags:     [theory, automata, computation, push-down automata, stack, context-free languages, context-free grammar, context-free]
---

Hello again! I'm picking up my [series on Automata]({% post_url 2016-03-28-theory-of-computation %}), with this post that goes into what I had always meant to get to: parsers. We'll check out the old-school linear time parsing algorithms, which only need to go over the input once, without backtracking or caching. We'll check out LL and LR, parse tables, recursive descent and recursive _ascent_. Welcome to the world of deterministic parsing...

# Refresher from Pushy Automata

We'll start with a brief refresher from the previous post of the series, [pushy automata]({% post_url 2016-05-15-pushy-automata %}), since that was a little while back.

## Push-down Automata

Push-down automata (PDAs) are automata with a _stack_. They don't just consume input and have fixed memory in their states, they can remember things on that single stack too, by pushing onto it and popping from it. Here's a deterministic PDA for recognising the language of words that start with zeroes, followed by an equal number of ones:

{% digraph Non-regular language example, deterministic %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;
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

## Context-Free Grammars, Derivations, and a naive PDA translation

A context-free grammar that describes the above language is:

:- | :-
{%latex%} S = 0 S 1 {%endlatex%}       | {%latex%} \text{(step)} {%endlatex%}
{%latex%} S = \varepsilon {%endlatex%} | {%latex%} \text{(}\varepsilon\text{)} {%endlatex%}

Sort {%latex%}S{%endlatex%} is the start symbol, the starting point in the grammar. If we're using the grammar _productively_ we start from the start symbol and use the rules left-to-right to replace sorts until we get the sentence in the language that we want. Something like: {%latex%} S \to 0 S 1 \to 0 0 S 1 1 \to 0 0 1 1 {%endlatex%}. This is called a _derivation_.

The most general, naive way to derive a push-down automaton for any context-free grammar is by pushing the end-of-stack and start symbol at the start, having a "main" state that uses the grammar rules with the body reversed (to deal with the stack order), and an accept state that pops the end-of-stack:

{% digraph Non-regular language example, deterministic %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;
start [shape=none, label="", width=0];
q₁ [shape=doublecircle, width=0.4];
start -> q₀ [label="$ S"];
q₀ -> q₁ [label="ε, $ → ε"];
q₀ -> q₀ [label="ε, S → 1 S 0\nε, S → ε\n0, 0 →ε\n1, 1 → ε"];
{% enddigraph %}

Here the stack grows left-to-right, so the lowest symbol on the stack is $ (end of stack), followed by S (the grammar start symbol). By the rules of the grammar we can manipulate the top of the stack and rewrite it to the body. If the input lines up with what we have on the stack, we can eliminate both. It's simple, but inefficient because of all the nondeterminism.

## Derivations, Parse Trees and Ambiguity

Let's look at a slightly more interesting grammar from a parser perspective:

:- | :-
{%latex%} S = S + S {%endlatex%} | {%latex%} \text{(add)} {%endlatex%}
{%latex%} S = S * S {%endlatex%} | {%latex%} \text{(mul)} {%endlatex%}
{%latex%} S = 1     {%endlatex%} | {%latex%} \text{(}\varepsilon\text{)} {%endlatex%}

When you want to derive {%latex%} 1 + 1 * 1 {%endlatex%}, you can do this in all manner of ways. The following derivation picks just an arbitrary sort on which to apply a rule from the grammar:

1. {%latex%} S {%endlatex%} (first S)
2. {%latex%} S + S {%endlatex%} (first S)
3. {%latex%} 1 + S {%endlatex%} (first S)
4. {%latex%} 1 + S * S {%endlatex%} (second S)
5. {%latex%} 1 + S * 1 {%endlatex%} (first S)
6. {%latex%} 1 + 1 * 1 {%endlatex%}

Notice how in some steps the leftmost {%latex%}S{%endlatex%} was replaced, while in others the rightmost was replaced. Generally speaking, you'll want either a leftmost or a rightmost derivation for parsers, which is to say: a grammar rule is always applied to the leftmost or rightmost sort. There are three reasons for this. The first is that you want a parser to be predictable in when it applies grammar rules, as you may connect so-called _semantic actions_ to each rule. These are pieces of code that are run when the parser applies the particular rule. (A typical example is a simple calculator). Such actions could perform side-effects, therefore order matters. For this reason, leftmost vs rightmost can also be observed. Two other reasons you to want this predictable derivation order is ease of implementation, and ease of proving things about your algorithm. These last two care less for whether it's leftmost or rightmost.

The most common semantic actions I'm aware of is to build a syntax tree with a parser. This builds a tree structure out of the parsed text. A parse tree, or concrete syntax tree, contains all the rule applications as seen in the grammar. An abstract syntax tree abstracts over some parts of the syntax tree, such as leaving out whitespace, or parentheticals (the shape of the tree captures the precedence anyway), or injections (grammars rules of the form {%latex%} S₁ = S₂ {%endlatex%}). Let's look at some parse trees of the last example, {%latex%} 1 + 1 * 1 {%endlatex%}:

{% digraph Parse trees of 1 + (1 * 1) and (1 + 1) * 1 %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101", label=S];
edge [fontcolor="#010101", color="#010101"];
rankdir=TB;
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

**With that recap out of the way:** For the purposes of _this_ blog post, we'll look at <em>un</em>ambiguous grammars for the languages we want to parse. Still, whether you use leftmost derivation or rightmost derivation in a parser that parses unambiguous grammars matters quite a lot in terms of what languages you can describe deterministically. It also influences how easily you can write a parser by hand for such a grammar, and how easily you can (programmatically) explain why your parser _doesn't_ accept certain inputs (parser error messages). So let's have a look at LL and LR parsing techniques, where the first L in those abbreviations stands for Left-to-right (as in reading direction in text), and the second letters are respectively leftmost and rightmost derivation.

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
{: .parsetable}

A table like the above is an LL(1) parse table, because it uses only 1 symbol of "look-ahead" in the columns. LL(1) grammars are always strong LL grammars, which means that they only need the combination of the sort to be parsed and the next symbol(s) to decide on a unique grammar rule to apply. In general, LL(k) grammars do not have to be strong, and if they are not, you also need to know what was already parsed from the input in order to choose a unique grammar rule[^LLdef]. For example, the following grammar is LL(2), and not strong:

:- | :-
{%latex%} S = A\ a\ b\ A\ b\ a {%endlatex%} | {%latex%} \text{(1)} {%endlatex%}
{%latex%} A = a {%endlatex%}                | {%latex%} \text{(2)} {%endlatex%}
{%latex%} A = {%endlatex%}                  | {%latex%} \text{(3)} {%endlatex%}

You can see this if you try to write an LL(2) parse table for it:

|                        | `a a` | `a b` | `b a` |
| :--------------------- | ----: | ----: | ----: |
| {%latex%}S{%endlatex%} |     1 |     1 |       |
| {%latex%}A{%endlatex%} |     2 |   2,3 |     3 |
{: .parsetable}

If you look ahead to `a b` on the input, and the next sort is {%latex%}A{%endlatex%}, then it really depends on whether you are at the start of the input or in the middle of rule 1. If you're at the start, you must choose rule 3 so you can parse `a b` as part of the rule 1, but if you're already in the middle of rule 1, you must choose rule 2 for {%latex%}A{%endlatex%} so you can continue to parse `b a` of rule 1.

If you mark {%latex%}A{%endlatex%} in rule 1 with where you are in rule 1 ({%latex%} S = A₁\ a\ b\ A₂\ b\ a {%endlatex%}), you get an LL(2) grammar that is strong, although the table for it is larger[^table]:

|                          | `a a` | `a b` | `b a` |
| :----------------------- | ----: | ----: | ----: |
| {%latex%}S{%endlatex%}   |     1 |     1 |       |
| {%latex%}A_1{%endlatex%} |     2 |     3 |       |
| {%latex%}A_2{%endlatex%} |       |     2 |     3 |
{: .parsetable}

In general, you can always use this trick to construct a strong, _structurally equivalent_ LL grammar with the same look-ahead. This is quite useful for constructing simple LL parsers. However, the downside of these parsers is that on wrong input they can fail later than a more complicated LL(k) parser that works for the non-strong grammar. And that matters if you want to give nice error messages.

### An intuition for table construction by automaton

Building the above tables was a matter of keeping in mind what they mean, and squinting a little. But in the case of a larger grammar, or a parsetable generator, of course you want an exact process. Before I dive into _First_ and _Follow_ sets that are the traditional method for building these tables, let me give you a method that is less practical but in my opinion more intuitive.

Step 1: Let's build a simple automaton for each rule of the grammar, where we assume both sorts and terminals are on the input. 

{% digraph Simple automata for each grammar rule from the last example %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;
subgraph {
start1 [shape=none, label="", width=0];
S₁ [shape=doublecircle, width=0.4];
start1 -> S₁₀;
S₁₀ -> S₁₁ [label="A"];
S₁₁ -> S₁₂ [label="a"];
S₁₂ -> S₁₃ [label="b"];
S₁₃ -> S₁₄ [label="A"];
S₁₄ -> S₁₅ [label="b"];
S₁₅ -> S₁ [label="a"];
}
subgraph {
node [style=filled, fillcolor="#ddd"];
start2 [style="", shape=none, label="", width=0];
A₂ [shape=doublecircle, width=0.4];
start2 -> A₂₀;
A₂₀ -> A₂ [label="a"];
}
subgraph {
node [style=filled, fillcolor="#aaa"];
start3 [style="", shape=none, label="", width=0];
A₃ [shape=doublecircle, width=0.4];
start3 -> A₃;
}
{% enddigraph %}

Note how each node of a rule automaton has the number of of the rule followed by the offset into the body of the rule. This is where we are in parsing by that rule, when that is our current state. The last node doesn't have this offset so you can easily identify it, when when it's no longer a final state.

Typically you'll find texts on parsers display the position in a rule more visually with "LR item" notation. This uses the actual rule and a dot to indicate where we are in the rule. While this makes individual automata states more readable, it makes the automata large and therefore harder to display in a readable way as a whole. Here's an example of the notation:


| Shorthand in this blog   | LR Item notation                                   |
| :----------------------- | :------------------------------------------------- |
| S₁₀                      | {%latex%} S = .\ A\ a\ b\ A\ b\ a {%endlatex%} |
| S₁₁                      | {%latex%} S = A\ .\ a\ b\ A\ b\ a {%endlatex%} |
| S₁₂                      | {%latex%} S = A\ a\ .\ b\ A\ b\ a {%endlatex%} |
| S₁₃                      | {%latex%} S = A\ a\ b\ .\ A\ b\ a {%endlatex%} |
| S₁₄                      | {%latex%} S = A\ a\ b\ A\ .\ b\ a {%endlatex%} |
| S₁₅                      | {%latex%} S = A\ a\ b\ A\ b\ .\ a {%endlatex%} |
| S₁                       | {%latex%} S = A\ a\ b\ A\ b\ a\ . {%endlatex%} |
{: .parsetable}

Step 2: Now instead of consuming a sort with an automaton, use {%latex%}\varepsilon{%endlatex%} rules to jump to the automata of the rules for that sort instead. Use the PDA stack with unique labels to get back to where you would be after consuming the sort.

{% digraph Single PDA using the automata from the grammar rules %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;

start1 [shape=none, label="", width=0];
S₁ [shape=doublecircle, width=0.4];
subgraph {
rank=same;
edge [style=invis];
A₂₀ [style=filled, fillcolor="#ddd"];
S₁₀ -> A₂₀;
}
subgraph {
rank=same;
edge [style=invis];
A₂ [style=filled, fillcolor="#ddd"];
A₃ [style=filled, fillcolor="#aaa"];
A₃ -> S₁₁;
S₁₁ -> A₂;
}
subgraph {
start1 -> S₁₀;
S₁₀ -> S₁₁ [label="A", fontcolor="#01010140", color="#01010140"];
S₁₁ -> S₁₂ [label="a"];
S₁₂ -> S₁₃ [label="b"];
S₁₃ -> S₁₄ [label="A", fontcolor="#01010140", color="#01010140"];
S₁₄ -> S₁₅ [label="b"];
S₁₅ -> S₁ [label="a"];
}

subgraph {
A₂₀ -> A₂ [label="a"];
}

S₁₀ -> A₂₀ [taillabel="↓S₁₁", labelangle=70, labeldistance=1.5, constraint=false];
A₂ -> S₁₁ [taillabel="↑S₁₁", labelangle=-60, labeldistance=2, constraint=false];
S₁₀ -> A₃ [taillabel="↓S₁₁", labelangle=60, labeldistance=1.75, constraint=false];
A₃ -> S₁₁ [taillabel="↑S₁₁", labelangle=60, labeldistance=2, constraint=false];

S₁₃ -> A₂₀ [taillabel="↓S₁₄", labelangle=-35, labeldistance=3, constraint=false];
A₂ -> S₁₄ [taillabel="↑S₁₄", labelangle=40, labeldistance=2, constraint=false];
S₁₃ -> A₃ [taillabel="↓S₁₄", labelangle=-50, labeldistance=2, constraint=false];
A₃ -> S₁₄ [taillabel="↑ S₁₄", labelangle=40, labeldistance=2, constraint=false];
{% enddigraph %}

The {%latex%}\downarrow{}X{%endlatex%} is an abbreviation for an {%latex%}\varepsilon, \varepsilon \rightarrow X{%endlatex%} edge that pushes a symbol on the stack unconditionally, it was hard to get graphviz to cooperate on node placement of this graph otherwise... Now at every node that had a sort transition you have multiple transition options, these are where you need to look ahead. So:

Step 3: at a sort transition node, for each {%latex%}\downarrow{%endlatex%} transition, follow transitions until you've consumed _k_ terminals (2 in this example) from the input. These are the terminals of the column in the parse table, and the rule of the {%latex%}\downarrow{%endlatex%} transition gets put into that cell. You can also put the look-ahead into the automaton:

{% digraph Single PDA using the automata from the grammar rules %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;

start1 [shape=none, label="", width=0];
S₁ [shape=doublecircle, width=0.4];
subgraph {
rank=same;
edge [style=invis];
A₂₀ [style=filled, fillcolor="#ddd"];
S₁₀ -> A₂₀;
}
subgraph {
rank=same;
edge [style=invis];
A₂ [style=filled, fillcolor="#ddd"];
A₃ [style=filled, fillcolor="#aaa"];
A₃ -> S₁₁;
S₁₁ -> A₂;
}
subgraph {
start1 -> S₁₀;
S₁₀ -> S₁₁ [label="A", fontcolor="#01010140", color="#01010140"];
S₁₁ -> S₁₂ [label="a"];
S₁₂ -> S₁₃ [label="b"];
S₁₃ -> S₁₄ [label="A", fontcolor="#01010140", color="#01010140"];
S₁₄ -> S₁₅ [label="b"];
S₁₅ -> S₁ [label="a"];
}

subgraph {
A₂₀ -> A₂ [label="a"];
}

S₁₀ -> A₂₀ [taillabel="(aa), ↓S₁₁", labelangle=0, labeldistance=1.5, constraint=false];
A₂ -> S₁₁ [taillabel="↑S₁₁", labelangle=-60, labeldistance=2, constraint=false];
S₁₀ -> A₃ [taillabel="(ab), ↓S₁₁", labelangle=80, labeldistance=2.5, constraint=false];
A₃ -> S₁₁ [taillabel="↑S₁₁", labelangle=60, labeldistance=2, constraint=false];

S₁₃ -> A₂₀ [taillabel="(ab), ↓S₁₄", labelangle=-30, labeldistance=5, constraint=false];
A₂ -> S₁₄ [taillabel="↑S₁₄", labelangle=40, labeldistance=2, constraint=false];
S₁₃ -> A₃ [taillabel="(ba), ↓S₁₄", labelangle=-60 labeldistance=2, constraint=false];
A₃ -> S₁₄ [taillabel="↑ S₁₄", labelangle=40, labeldistance=2, constraint=false];
{% enddigraph %}

### Building LL tables for strong LL grammars by traditional method

While the above building of automata gives a visual intuition, it's not the most efficient way to express how we can build LL tables. The traditional method does the same thing in essence, but using some pre-computed sets to calculate the cells.

A cell in the table at the row labeled with sort {%latex%}A{%endlatex%} and the column labeled with terminal(s) {%latex%}v{%endlatex%} should have the grammar rule {%latex%}A = w{%endlatex%} (where {%latex%}w{%endlatex%} is a mix of terminals and sorts or {%latex%}\varepsilon{%endlatex%}), under the following condition: {%latex%}v{%endlatex%} is in the _First_ set of {%latex%}w{%endlatex%}, or {%latex%}\varepsilon{%endlatex%} is in the _First_ set of {%latex%}w{%endlatex%} and {%latex%}v{%endlatex%} is in the _Follow_ set of {%latex%}A{%endlatex%}. In other words: {%latex%}v \in \textit{First}(w) \cdot \textit{Follow}(A){%endlatex%}

Let's unpack that. The _First_ set of a terminal is a singleton set with that terminal. The _First_ set of a sort is the set of first non-terminals that the sort can expand to, directly or indirectly. So a rule {%latex%}A = a \textrm{\footnotesize[...]}{%endlatex%} causes {%latex%}a{%endlatex%} to appear in the _First_ set of {%latex%}A{%endlatex%}, {%latex%}A = B \textrm{\footnotesize[...]}{%endlatex%} causes the _First_ set of {%latex%}B{%endlatex%} to also be in the _First_ set of {%latex%}A{%endlatex%}, and {%latex%}A = \varepsilon{%endlatex%} causes {%latex%}\varepsilon{%endlatex%} to appear in the _First_ set of {%latex%}A{%endlatex%}. This last rule says {%latex%}A{%endlatex%} can be expanded to "nothing", so if that's an option we need to check the _Follow_ set of {%latex%}A{%endlatex%}.

The _Follow_ set is basically every non-terminal that can follow {%latex%}A{%endlatex%} in the grammar. So when you have {%latex%}B = \textrm{\footnotesize[...]} A\ a \textrm{\footnotesize[...]}{%endlatex%}, {%latex%}a{%endlatex%} is in the follow set of {%latex%}A{%endlatex%}. A rule {%latex%}B = \textrm{\footnotesize[...]} A{%endlatex%} causes the _Follow_ set of {%latex%}B{%endlatex%} to be in the _Follow_ set of {%latex%}A{%endlatex%}. And the _Follow_ set of the start symbol has the end-of-file meta-terminal.

Finally, there is the dot operator between the _First_ and _Follow_ sets: this is a _truncated product_, that takes every combination of the two sets, sticks them together (in order), and truncates to length k. That's a bit of an abstraction over the k in LL(k), which I didn't take into account in the explanation of _First_ and _Follow_ sets. The _First_ sets should have length k strings of course, and so you may need to take more _First/Follow_ sets into account when computing these. Another thing I glossed over is that we actually use the _First_ set of {%latex%}w{%endlatex%}, a mix of terminals and sorts on the right-hand side of our grammar rules. If {%latex%}w{%endlatex%} is {%latex%}v\ A\ B\ x{%endlatex%}, then its _First_ set is {%latex%}\{v\} \cdot \textit{First}(A) \cdot \textit{First}(B) \cdot \{x\}{%endlatex%}.

Ok, with that all done, we can use those tables. But first we need to talk expressive power, because LL is not particularly powerful...

### Limitations and Expressive power

There are always languages that cannot be captured by an LL(k) grammar that can be captured by an LL(k+1) grammar. In other words, look-ahead size is important in the expressivity of an LL grammar, and LL(k) for any specific k does not capture _all_ context-free languages.

We've already seen an example of an LL(2) grammar, but what would be a language that cannot be captured by any LL(k)? Take the language of a's followed by b's, where the number of a's {%latex%}\geq{%endlatex%} the number of b's. Or as a grammar:

:- | :-
{%latex%} S = a S         {%endlatex%} | {%latex%} \text{(1)} {%endlatex%}
{%latex%} S = A           {%endlatex%} | {%latex%} \text{(2)} {%endlatex%}
{%latex%} A = a A b       {%endlatex%} | {%latex%} \text{(3)} {%endlatex%}
{%latex%} A = \varepsilon {%endlatex%} | {%latex%} \text{(4)} {%endlatex%}

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

Our parse table has now become code directly, with these functions named after the sorts of the rows. They call rules that are also functions, which in turn use the sort functions. Those rules also use `consume`, this time without having to reverse any orders.

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

We've seen how we can construct simple DFAs for each rule in our grammar, and then replace the sort transitions {%latex%}N_1 \xrightarrow{A} N_2{%endlatex%} by a (PDA) push transition ({%latex%}\downarrow A{%endlatex%}) from {%latex%}N_1{%endlatex%} to all starts of DFAs corresponding to rules of {%latex%}A{%endlatex%}, and a pop transition ({%latex%}\uparrow A{%endlatex%}) from the ends of those DFAs to {%latex%}N_2{%endlatex%}. Then the LL table, the decision table of sort + look-ahead = rule, naturally follows from this PDA by finding what input will be consumed if a certain rule is chosen, and using that as the look-ahead to make the decision for that rule.

The recursive descent way of writing a parser directly as code is nice and simple, it really just follows the grammar. Since you're writing plain old code with function calls, you can imagine people have found nice ways to extend and adapt the pattern of recursive descent parsers. For one, it's quite easy to reason about where you are in the parse when hitting an error state, which makes it fairly easy to give friendly error messages when the parser doesn't accept an input. You can also use a trick to fix up direct left-recursion called [node reparenting](https://en.wikipedia.org/wiki/Tail_recursive_parser), where you use a loop or tail-recursion locally construct the tree bottom-up. You could argue that such a parser is a hybrid between recursive descent and ascent, a "recursive descent-ascent parser".

Finally, if we look back at the automaton, we can see that the PDAs we build have a very strict shape. We either have a non-deterministic choice due to multiple push transitions for a sort, or we have predicted input, a single path of terminals to consume from the input. If we think back to the [NFAs and DFAs]({% post_url 2016-04-10-finite-automata %}) from early on in this blog post series, those used the input to chose what state to go to next. Now we have single-path DFAs that just consume input, and a separate table on a look-ahead to resolve non-determinism from the pushes and pops. The table is really just another DFA, but one that doesn't consume the input, just looks at it. The strict shape here indicated that we're not really making full use of the power of automata. This will change with the next parsing technique.

# Bottomup, LR parsing

LR stands for left-to-right, rightmost derivation in reverse. If you think about it, left-to-right and rightmost derivation are incompatible: The rightmost derivation chooses the rule for the rightmost sort first every time, but that means skipping over some unknown amount of input if you read left-to-right to even get to that point. However, the _reverse_ of the rightmost derivation is a left-to-right form of parsing. This reverse derivation describes going bottomup, left-to-right through the parse tree.

## Expressive power and relation to LL

One of the biggest upsides of LR(k) parsing is its __expressivity__. The set of all LL(k) languages of any _k_ is a strict subset of all LR(1) languages. Note that this is speaking of languages, not grammars. For grammars it holds that any LL(k) grammar for a specific _k_ is also an LR(k) grammar, and not necessarily the other way around.

An LR(k) grammar of any k greater than 1 can be automatically transformed into an LR(1) grammar that is not necessarily structurally equivalent. This is highlights the difference between grammar and language level equivalence. We can basically capture any LR language in an LR(1) grammar, but LR with larger _k_ may be able to describe the language in a nicer way (smaller grammar).

A good overview of how LL and LR relate to each other on the grammar and language level is [summarised on the Computer Science Stack Exchange](https://cs.stackexchange.com/a/48). In the comments someone suggests making a list of examples for each of these relationships, which seems like a great idea, but not something I have the patience for right now. This blog post has enough scope creep already.

## How LR works

In order to give a reverse rightmost derivation, we need to figure what sorts can be at the leftmost leaf of the parse tree for our LR grammar. Then we try to apply the rules for those sorts all simultaneously. And to do so we can't just use the automaton build we've used for LL.

Remember that the automata we've used previously mapped well on recursive descent, and showed us where to use an LL parse table with look-ahead to resolve ambiguity. Crucially, those automata observe every rule we go into. But for LR we need to explore down all the rules simultaneously. Let's see if we can't get somewhere with that idea and the example grammar of the language that wasn't LL:

:- | :-
{%latex%} S = a S         {%endlatex%} | {%latex%} \text{(1)} {%endlatex%}
{%latex%} S = A           {%endlatex%} | {%latex%} \text{(2)} {%endlatex%}
{%latex%} A = a A b       {%endlatex%} | {%latex%} \text{(3)} {%endlatex%}
{%latex%} A = \varepsilon {%endlatex%} | {%latex%} \text{(4)} {%endlatex%}

We start again with the separate automata for each rule:

{% digraph Simple automata for each grammar rule from the example %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;
subgraph {
node [style=filled, fillcolor="#777", fontcolor="#fefefe"];
start4 [style="", shape=none, label="", width=0];
A₄ [shape=doublecircle, width=0.4];
start4 -> A₄;
}
subgraph {
node [style=filled, fillcolor="#aaa"];
start3 [style="", shape=none, label="", width=0];
A₃ [shape=doublecircle, width=0.4];
start3 -> A₃₀;
A₃₀ -> A₃₁ [label="a"];
A₃₁ -> A₃₂ [label="A"];
A₃₂ -> A₃ [label="b"];
}
subgraph {
node [style=filled, fillcolor="#ddd"];
start2 [style="", shape=none, label="", width=0];
S₂ [shape=doublecircle, width=0.4];
start2 -> S₂₀;
S₂₀ -> S₂ [label="A"];
}
subgraph {
start1 [shape=none, label="", width=0];
S₁ [shape=doublecircle, width=0.4];
start1 -> S₁₀;
S₁₀ -> S₁₁ [label="s"];
S₁₁ -> S₁ [label="A"];
}
{% enddigraph %}

Now in order to explore to the bottom-left of the parse tree, we need to be free to go into any rule. So we will connect the rules again to the nodes that expect a certain sort, but with epsilon transitions so we don't observe how far down we are or with what rule in particular we got there. We'll need that later, but let's not worry about that until we have the downward exploration.

{% digraph Partially constructed automaton using the automata from the grammar rules, using epsilon transitions %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;
start1 [shape=none, label="", width=0];
start2 [shape=box, label="", width=0, color="#00000000"];
start3 [shape=none, label="", width=0];
subgraph {
node [style=filled, fillcolor="#777", fontcolor="#fefefe"];
A₄ [shape=doublecircle, width=0.4];
}
subgraph {
node [style=filled, fillcolor="#aaa"];
A₃ [shape=doublecircle, width=0.4];
start3 -> A₃₀ [style=invis, weight=3];
A₃₀ -> A₃₁ [label="a", weight=10];
A₃₁ -> A₃₂ [label="A", weight=3];
A₃₂ -> A₃ [label="b", weight=3];
}
subgraph {
node [style=filled, fillcolor="#ddd"];
S₂ [shape=doublecircle, width=0.4];
start1 -> start2:w [arrowhead=none];
start2:e -> S₂₀ [weight=3];
S₂₀ -> S₂ [label="A", weight=3];
}
subgraph {
S₁ [shape=doublecircle, width=0.4];
start1 -> S₁₀ [weight=3];
S₁₀ -> S₁₁ [label="a", weight=3];
S₁₁ -> S₁ [label="S", weight=3];
}
subgraph{
rank=same;
edge [style=invis];
start1 -> start3 [minlen=3, weight=3];
}
subgraph{
rank=same;
edge [style=invis];
S₁₀ -> start2 [weight=3];
start2 -> A₃₀ [weight=3, minlen=2];
}
subgraph{
rank=same;
edge [style=invis];
S₁₁ -> S₂₀ [weight=3];
S₂₀ -> A₃₁ [weight=3, minlen=2];
A₃₁ -> A₄ [weight=3];
}
subgraph{
rank=same;
edge [style=invis];
S₁ -> S₂ [weight=3];
S₂ -> A₃₂ [weight=3, minlen=2];
}
S₁₁ -> S₂₀ [label="ε"];
S₁₁ -> S₁₀ [label="ε"];
S₂₀ -> A₃₀ [label="ε"];
A₃₁ -> A₄ [label="ε"];
A₃₁ -> A₃₀:se [label="ε"];
S₂₀ -> start3:n [arrowhead="none"];
start3:s -> A₄ [label="ε"];
{% enddigraph %}

Obviously this is not a full automaton model of a parser yet, but it allows us to always go down to the next leaf of the parse tree without using the stack. Let's clean up the automaton with an NFA-to-DFA conversion:

{% digraph Partially constructed automaton using the automata from the grammar rules, after merging states through NFA-to-DFA conversion %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;

start1 [shape=none, label="", width=0];
S₁ [shape=doublecircle, width=0.4];
S₂ [shape=doublecircle, width=0.4, style=filled, fillcolor="#ddd"];
A₃ [shape=doublecircle, width=0.4, style=filled, fillcolor="#aaa"];
Box0 [shape=none, label=<<table cellborder="0" port="t"><tr><td>S₁₀</td></tr><tr><td bgcolor="#ddd">S₂₀</td></tr><tr><td bgcolor="#aaa">A₃₀</td></tr><tr><td border="1" bgcolor="#777"><font color="#fefefe">A₄</font></td></tr></table>>];
Box1 [shape=none, label=<<table cellborder="0" port="t"><tr><td>S₁₀</td></tr><tr><td>S₁₁</td></tr><tr><td bgcolor="#ddd">S₂₀</td></tr><tr><td bgcolor="#aaa">A₃₀</td></tr><tr><td bgcolor="#aaa">A₃₁</td></tr><tr><td border="1" bgcolor="#777"><font color="#fefefe">A₄</font></td></tr></table>>];
Box2 [shape=none, label=<<table cellborder="0" port="t"><tr><td border="1" bgcolor="#ddd">S₂</td></tr><tr><td bgcolor="#aaa">A₃₂</td></tr></table>>];
start1 -> Box0:t;
Box0:t -> Box1:t [label="a", weight=2];
Box0:t -> S₂ [label="A"];
Box1:t:n -> Box1:t:n [label="a"];
Box1:t -> Box2:t [label="A", weight=2];
Box1:t -> S₁ [label="S"];
Box2:t -> A₃ [label="b"];

subgraph {
rank=same;
edge [style=invis];
Box2:t -> S₁ [minlen=0];
}
subgraph {
rank=same;
edge [style=invis];
Box1:t -> S₂ [minlen=0];
}
{% enddigraph %}

This is almost exactly how an LR(0) parse table would be drawn. Instead of S₁₀ and S₁₁, you write out "LR item"s like `S = . a S` and `S = a . S`, and this would allow you to leave off the extra borders on all the fully finished rules because they would be recognisable by the dot at the end. But otherwise it would be pretty much this. That's because what's actually happening on the stack is very structured, but a little involved. That makes the PDA harder to read and draw, but I'll demonstrate it once:

{% digraph A fully explicit PDA that does LR parsing %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;

start1 [shape=none, label="", width=0];
fin [shape=doublecircle, width=0.4, label=""]
S₁;
S₂ [style=filled, fillcolor="#ddd"];
A₃ [style=filled, fillcolor="#aaa"];
Box0 [shape=none, label=<<table cellborder="0" port="t"><tr><td>S₁₀</td></tr><tr><td bgcolor="#ddd">S₂₀</td></tr><tr><td bgcolor="#aaa">A₃₀</td></tr><tr><td bgcolor="#777"><font color="#fefefe">A₄</font></td></tr></table>>];
Box1 [shape=none, label=<<table cellborder="0" port="t"><tr><td>S₁₀</td></tr><tr><td>S₁₁</td></tr><tr><td bgcolor="#ddd">S₂₀</td></tr><tr><td bgcolor="#aaa">A₃₀</td></tr><tr><td bgcolor="#aaa">A₃₁</td></tr><tr><td bgcolor="#777"><font color="#fefefe">A₄</font></td></tr></table>>];
Box2 [shape=none, label=<<table cellborder="0" port="t"><tr><td bgcolor="#ddd">S₂</td></tr><tr><td bgcolor="#aaa">A₃₂</td></tr></table>>];
start1 -> Box0:t;
Box0:t -> Box1:t [label="a,↓[a,0]", weight=3];
Box0:t -> S₂ [headlabel="↓[A,0]", labelangle=30, labeldistance=5];
Box1:t:n -> Box1:t:n [label="a,↓[a,1]"];
Box1:t -> Box2:t [label="↓[A,1]", weight=3];
Box2:t -> A₃ [label="b,↓[b,2]", weight=3];

S₁:e -> S₁:e [taillabel=" ↑[S,1] [a,1] ↓[S,1]", labeldistance=5, labelangle=-15];
S₁ -> fin [headlabel="↑[S,1] [a,0]", labeldistance=4.5, labelangle=-30];
S₂ -> fin [label="↑[A,0]     "]
Box2:t -> S₁:n [xlabel="\n\n↑[A,1] ↓[S,1]  "];
A₃ -> Box2:t [xlabel="↑[b,2] [A,1] [a,1] ↓[A,1]"];
A₃ -> S₂ [taillabel="↑[b,2] [A,1] [a,0] ↓[A,0]", labeldistance=5, labelangle=40];

subgraph {
rank=same;
edge [style=invis];
Box2:t -> S₁ [minlen=4];
}
subgraph {
rank=same;
edge [style=invis];
Box1:t -> S₂ [minlen=0];
S₂ -> fin [minlen=0];
}
{% enddigraph %}

This should look quite familiar. We're pushing inputs on the stack as we consume them. And then we're popping whole bodies of rules off the stack and replacing them with the sort of that rule. The first thing is called a _shift_ transition, the second is called _reduce_ transition. We've seen this trick before in the naive PDA built from a CFG, all the way at the start of this post in the refresher.

Notice that _where_ a reduce transition goes depends on originating state of the last thing that's popped. That's why we keep track of those on the stack. When we reduce by rule 3 (state A₃), depending on whether the `a` came from box 1 or box 0, we go to different places. This is easier to see in our LR automaton, where box 1 points to state S₁ with a transition labeled `A`. This is a _goto_ transition. In an LR parse table interpreter, the _goto_ is a separate action that immediately follows a _reduce_ action, just returns to the last popped state. That's also why a reduce transition in the above automaton always labels the originating state of the pushed sort the same as the last thing popped from the stack.

Something worth repeating now that we're looking at the details: LL decides what rule to take _before_ consuming the input for that rule, whereas LR decides what rule to take _after_ consuming all the input for that rule. In other words, we only reduce by a rule when we've seen the entire body of the rule, that why there's less trouble with look-ahead.

Speaking of look-ahead: we have some shift-reduce problems in our automaton. And by that I mean: how do we choose when to shift and when to reduce when both are an option? This is a determinism issue in our current automaton, and just like in our LL automaton, we solve it with look-ahead (and yes, that can and will later be summarised in a parse table). Our latest automaton gives a clear view of what we will be able to do if we reduce, so the look-ahead follows from what input can be consumed next after each reduce:

{% digraph A fully explicit PDA that does LR parsing, with look-ahead %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;

start1 [shape=none, label="", width=0];
fin [shape=doublecircle, width=0.4, label=""];
S₁;
S₂ [style=filled, fillcolor="#ddd"];
A₃ [style=filled, fillcolor="#aaa"];
Box0 [shape=none, label=<<table cellborder="0" port="t"><tr><td>S₁₀</td></tr><tr><td bgcolor="#ddd">S₂₀</td></tr><tr><td bgcolor="#aaa">A₃₀</td></tr><tr><td bgcolor="#777"><font color="#fefefe">A₄</font></td></tr></table>>];
Box1 [shape=none, label=<<table cellborder="0" port="t"><tr><td>S₁₀</td></tr><tr><td>S₁₁</td></tr><tr><td bgcolor="#ddd">S₂₀</td></tr><tr><td bgcolor="#aaa">A₃₀</td></tr><tr><td bgcolor="#aaa">A₃₁</td></tr><tr><td bgcolor="#777"><font color="#fefefe">A₄</font></td></tr></table>>];
Box2 [shape=none, label=<<table cellborder="0" port="t"><tr><td bgcolor="#ddd">S₂</td></tr><tr><td bgcolor="#aaa">A₃₂</td></tr></table>>];
start1 -> Box0:t;
Box0:t -> Box1:t [label="a,↓[a,0]", weight=3];
Box0:t -> S₂ [headlabel="($), ↓[A,0]", labelangle=30, labeldistance=5];
Box1:t:n -> Box1:t:n [label="a,↓[a,1]"];
Box1:t -> Box2:t [label="(b,$), ↓[A,1]", weight=3];
Box2:t -> A₃ [label="b,↓[b,2]", weight=3];

S₁:e -> S₁:e [taillabel=" ↑[S,1] [a,1] ↓[S,1]", labeldistance=5, labelangle=-15];
S₁ -> fin [headlabel="↑[S,1] [a,0]", labeldistance=4.5, labelangle=-30];
S₂ -> fin [label="↑[A,0]     "]
Box2:t -> S₁:n [xlabel="\n\n($), ↑[A,1] ↓[S,1]  "];
A₃ -> Box2:t [xlabel="↑[b,2] [A,1] [a,1] ↓[A,1]"];
A₃ -> S₂ [taillabel="↑[b,2] [A,1] [a,0] ↓[A,0]", labeldistance=5, labelangle=40];

subgraph {
rank=same;
edge [style=invis];
Box2:t -> S₁ [minlen=4];
}
subgraph {
rank=same;
edge [style=invis];
Box1:t -> S₂ [minlen=0];
S₂ -> fin [minlen=0];
}
{% enddigraph %}

As you can see, we need at most 1 look-ahead to deterministically parse this grammar. We're sometimes looking ahead to the end-of-input represented with `$`. The look-ahead makes this an LALR(1) grammar; LR(1) is slightly more involved as we'll see in the next section. 

## LR parsetable construction and expressivity

Let's look at some example grammars, how to construct their tables, and when you need a better parsetable construction method.

### LR(0)

LR(0) does not look ahead but just reduces whenever possible. If there are multiple options, you have a shift-reduce or a reduce-reduce conflict. Shift-shift conflicts don't exist in LR since the NFA-to-DFA process would have merged the two states such conflicting transitions would point to.
Let's look at an LR(0) grammar:

:- | :-
{%latex%} S = E 2         {%endlatex%} | {%latex%} \text{(1)} {%endlatex%}
{%latex%} E = E 1         {%endlatex%} | {%latex%} \text{(2)} {%endlatex%}
{%latex%} E = 1           {%endlatex%} | {%latex%} \text{(3)} {%endlatex%}

The LR automaton for this grammar is:

{% digraph An LR(0) automaton for the above grammar %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;

subgraph {
rank=same;
Box0 [shape=none, label=<<table cellborder="0" port="t"><tr><td>S₁₀</td></tr><tr><td bgcolor="#ddd">E₂₀</td></tr><tr><td bgcolor="#aaa">E₃₀</td></tr></table>>];
fin [shape=doublecircle, width=0.4, label=""];
Box0:t:s -> fin:n [xlabel="S  "];
}

start1 [shape=none, label="", width=0];
fin [shape=doublecircle, width=0.4, label=""];
S₁;
E₂ [style=filled, fillcolor="#ddd"];
E₃ [style=filled, fillcolor="#aaa"];
Box1 [shape=none, label=<<table cellborder="0" port="t"><tr><td>S₁₁</td></tr><tr><td bgcolor="#ddd">E₂₁</td></tr></table>>];
start1 -> Box0:t;
Box0:t -> Box1:t [label="E", weight=2, minlen=2];
Box0:t -> E₃ [label="1"];
Box1:t -> S₁ [label="2"];
Box1:t -> E₂ [label="1"];
{% enddigraph %}

The corresponding parse table follows this automaton:

|           | `1`  | `2`  | `$`      | `E`  |
|:----------|:----:|:----:|:--------:|:----:|
| **Box0**  | s E₃ |      | _accept_ | Box1 |
| **Box1**  | s E₂ | s S₁ |          |      |
| **E₃**    | r 3  | r 3  | r 3      |      |
| **E₂**    | r 2  | r 2  | r 2      |      |
| **S₁**    | r 1  | r 1  | r 1      |      |
{: .parsetable}

The transition from box 0 to E₃ that shifts `1` becomes a shift action to E₃ in the row of box 0 and the column of `1`. The transition from box 0 to box 1 with `E` becomes a goto to box 1 in the row of box 0 and column of `E`. Finally a state that's at the end of a rule will get all reduce actions by that rule (indicated by its number) in the column for input. Accepting the input is typically based on look-ahead of the end-of-input. 

### Simple LR (SLR)

The smallest motivating example for Simple LR is the following grammar that parses the same language as before:

:- | :-
{%latex%} S = E 2         {%endlatex%} | {%latex%} \text{(1)} {%endlatex%}
{%latex%} E = 1 E         {%endlatex%} | {%latex%} \text{(2)} {%endlatex%}
{%latex%} E = 1           {%endlatex%} | {%latex%} \text{(3)} {%endlatex%}

Notice how rule 2 is now right-recursive instead of left-recursive. It's a nice symmetry how left-recursive rules give you trouble in LL, and right-recursive rules give you trouble in LR[^indirect-recursion]. 

{% digraph An LR(0) automaton for the above grammar %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;

subgraph {
rank=same;
Box0 [shape=none, label=<<table cellborder="0" port="t"><tr><td>S₁₀</td></tr><tr><td bgcolor="#ddd">E₂₀</td></tr><tr><td bgcolor="#aaa">E₃₀</td></tr></table>>];
fin [shape=doublecircle, width=0.4, label=""];
Box0:t:s -> fin:n [xlabel="S  "];
}

start1 [shape=none, label="", width=0];
fin [shape=doublecircle, width=0.4, label=""];
S₁₁;
E₂ [style=filled, fillcolor="#ddd"];
Box1 [shape=none, label=<<table cellborder="0" port="t"><tr><td bgcolor="#ddd">E₂₀</td></tr><tr><td bgcolor="#ddd">E₂₁</td></tr><tr><td bgcolor="#aaa">E₃</td></tr></table>>];
start1 -> Box0:t;
Box0:t -> Box1:t [label="1", weight=2];
Box0:t -> S₁₁ [label="E"];
S₁₁ -> S₁ [label="2"];
Box1:t -> E₂ [label="E"];
Box1:t:s -> Box1:t:s [label="1"];
{% enddigraph %}

|           | `1`          | `2`  | `$`      | `E`  |
|:----------|:------------:|:----:|:--------:|:----:|
| **Box0**  | s Box1       |      | _accept_ | S₁₁  |
| **Box1**  | s Box1 / r 3 | r 3  | r 3      | E₂   |
| **S₁₁**   |              | s S₁ |          |      |
| **S₁**    | r 1          | r 1  | r 1      |      |
| **E₂**    | r 2          | r 2  | r 2      |      |
{: .parsetable}

Yay, we have a shift-reduce conflict. How do we solve it? By not blindly putting a reduce in the entire row of a state that could reduce. If we check the _Follow_ set of the sort we're reducing to (we defined that when we built LL parse tables, remember?), we can put the reduce action in only the column of the input symbols that are in that follow set. If we look at the grammar, we can see that only `2` can follow `E`. So the SLR table for this grammar is:

|           | `1`    | `2`  | `$`      | `E`  |
|:----------|:------:|:----:|:--------:|:----:|
| **Box0**  | s Box1 |      | _accept_ | S₁₁  |
| **Box1**  | s Box1 | r 3  |          | E₂   |
| **S₁₁**   |        | s S₁ |          |      |
| **S₁**    |        |      | r 1      |      |
| **E₂**    |        | r 2  |          |      |
{: .parsetable}

### Look-Ahead LR (LALR)

From now on we'll be looking at reduce-reduce conflicts only. While you can get shift-reduce conflicts with the following algorithms through grammars that don't fit (due to ambiguity or requiring more look-ahead than you're taking into account), when you give an LALR(k) grammar to an SLR(k) algorithm you can only get reduce-reduce conflicts. Same with an LR(k) grammar put through the LALR(k) algorithm.

Here our example grammar that just barely doesn't work with SLR:

:- | :-
{%latex%} S = a E c         {%endlatex%} | {%latex%} \text{(1)} {%endlatex%}
{%latex%} S = a F d         {%endlatex%} | {%latex%} \text{(2)} {%endlatex%}
{%latex%} S = b F c         {%endlatex%} | {%latex%} \text{(3)} {%endlatex%}
{%latex%} E = e             {%endlatex%} | {%latex%} \text{(4)} {%endlatex%}
{%latex%} F = e             {%endlatex%} | {%latex%} \text{(5)} {%endlatex%}

See how rules 4 and 5 are the same except they have different sort names? Yeah, that's going to be fun if they're used with the same prefix like in rules 1 and 2. Let's have a look at the automaton and SLR parse table.

{% digraph A LR(0) automaton for the above grammar %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;

subgraph {
rank=same;
Box0 [shape=none, label=<<table cellborder="0" port="t">
	<tr><td>S₁₀</td></tr><tr><td bgcolor="#ddd">S₂₀</td></tr>
	<tr><td>S₃₀</td></tr>
</table>>];
fin [shape=doublecircle, width=0.4, label=""];
Box0:t:s -> fin:n [xlabel="S  "];
}

start1 [shape=none, label="", width=0];
fin [shape=doublecircle, width=0.4, label=""];
S₁₂;
S₁;
S₂₂ [style=filled, fillcolor="#ddd"];
S₂ [style=filled, fillcolor="#ddd"];
S₃₂;
S₃;
F₅ [fontcolor="#fefefe", style=filled, fillcolor="#777"];
Box1 [shape=none, label=<<table cellborder="0" port="t">
	<tr><td>S₁₁</td></tr>
	<tr><td bgcolor="#ddd">S₂₁</td></tr>
	<tr><td bgcolor="#aaa">E₄₀</td></tr>
	<tr><td bgcolor="#777"><font color="#fefefe">F₅₀</font></td></tr>
</table>>];
Box2 [shape=none, label=<<table cellborder="0" port="t">
	<tr><td>S₃₁</td></tr>
	<tr><td bgcolor="#aaa">E₄₀</td></tr>
	<tr><td bgcolor="#777"><font color="#fefefe">F₅₀</font></td></tr>
</table>>];
Box3 [shape=none, label=<<table cellborder="0" port="t">
	<tr><td bgcolor="#aaa">E₄</td></tr>
	<tr><td bgcolor="#777"><font color="#fefefe">F₅</font></td></tr>
</table>>];
start1 -> Box0:t;
Box0:t -> Box1:t [label="a"];
Box0:t -> Box2:t [label="b"];
Box1:t -> Box3:t [label="e"];
Box1:t -> S₁₂ [label="E"];
Box1:t -> S₂₂ [label="F"];
S₁₂ -> S₁ [label="c"];
S₂₂ -> S₂ [label="d"];
Box2:t -> F₅ [label="e"];
Box2:t -> S₃₂ [label="F"];
S₃₂ -> S₃ [label="c"];
{% enddigraph %}

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
{: .parsetable}

The reduce-reduce conflict, as promised. It's in box 3 (E₄/F₅) when the look-ahead is `c`. This is because the look-ahead sets of both `E` and `F` contain `c` due to rules 1 and 3. If we look at the automaton though, we can clearly see that if we reduce and we have a `c` next, we should reduce by `E`.

Look-Ahead LR parsing uses basically this method, analysing what shifts can happen after certain reduces. Putting it is algorithmic terms, LALR doesn't use LL _Follow_ sets, but defines more accurate _Follow_ sets based on the automaton. Each instance of the start of a rule in the automaton (F₅₀ in boxes 1 and 2) gets a separate _Follow_ set computed. That's how we resolve the conflict with LALR:

|           | `a`    | `b`    | `c` | `d`  | `e`    | `$`      | `E`  | `F`  |
|:----------|:------:|:------:|:---:|:----:|:------:|:--------:|:----:|:----:|
| **Box3**  |        |        | r 4 | r 5  |        |          |      |      |
{: .parsetable}

Note that since the LALR _Follow_ sets follow directly from the automaton, this is basically the same as the intuition given at the end of the [previous section](#how-lr-works).

### LR(1)

I like this LALR parsing story. It's so intuitive with the NFA-to-DFA conversion, just looking at the automaton to see the follow sets. But, it's doesn't give you the complete power of deterministic push-down automata. I present to you the previous example grammar with one more rule:

:- | :-
{%latex%} S = a E c         {%endlatex%} | {%latex%} \text{(1)} {%endlatex%}
{%latex%} S = a F d         {%endlatex%} | {%latex%} \text{(2)} {%endlatex%}
{%latex%} S = b F c         {%endlatex%} | {%latex%} \text{(3)} {%endlatex%}
{%latex%} E = e             {%endlatex%} | {%latex%} \text{(4)} {%endlatex%}
{%latex%} F = e             {%endlatex%} | {%latex%} \text{(5)} {%endlatex%}
{%latex%} S = b E d         {%endlatex%} | {%latex%} \text{(6)} {%endlatex%}

You might think: "What's the problem? You've changed the LL _Follow_ set of E, but we have the more accurate LALR _Follow_ set to take care of that." You would be wrong, due to state merging in the NFA-to-DFA conversion:

{% digraph A LR(0) automaton for the above grammar %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;

subgraph {
rank=same;
Box0 [shape=none, label=<<table cellborder="0" port="t">
	<tr><td>S₁₀</td></tr><tr><td bgcolor="#ddd">S₂₀</td></tr>
	<tr><td>S₃₀</td></tr>
	<tr><td bgcolor="#ddd">S₆₀</td></tr>
</table>>];
fin [shape=doublecircle, width=0.4, label=""];
Box0:t:s -> fin:n [xlabel="S  "];
}

start1 [shape=none, label="", width=0];
fin [shape=doublecircle, width=0.4, label=""];
S₁₂;
S₁;
S₂₂ [style=filled, fillcolor="#ddd"];
S₂ [style=filled, fillcolor="#ddd"];
S₃₂;
S₃;
S₆₂ [style=filled, fillcolor="#ddd"];
S₆ [style=filled, fillcolor="#ddd"];
Box1 [shape=none, label=<<table cellborder="0" port="t">
	<tr><td>S₁₁</td></tr>
	<tr><td bgcolor="#ddd">S₂₁</td></tr>
	<tr><td bgcolor="#aaa">E₄₀</td></tr>
	<tr><td bgcolor="#777"><font color="#fefefe">F₅₀</font></td></tr>
</table>>];
Box2 [shape=none, label=<<table cellborder="0" port="t">
	<tr><td>S₃₁</td></tr>
	<tr><td bgcolor="#ddd">S₆₁</td></tr>
	<tr><td bgcolor="#aaa">E₄₀</td></tr>
	<tr><td bgcolor="#777"><font color="#fefefe">F₅₀</font></td></tr>
</table>>];
Box3 [shape=none, label=<<table cellborder="0" port="t">
	<tr><td bgcolor="#aaa">E₄</td></tr>
	<tr><td bgcolor="#777"><font color="#fefefe">F₅</font></td></tr>
</table>>];
start1 -> Box0:t;
Box0:t -> Box1:t [label="a"];
Box0:t -> Box2:t [label="b"];
Box1:t -> Box3:t [label="e"];
Box1:t -> S₁₂ [label="E"];
Box1:t -> S₂₂ [label="F"];
S₁₂ -> S₁ [label="c"];
S₂₂ -> S₂ [label="d"];
Box2:t -> Box3:t [label="e"];
Box2:t -> S₃₂ [label="F"];
S₃₂ -> S₃ [label="c"];
Box2:t -> S₆₂ [label="E"];
S₆₂ -> S₆ [label="d"];
{% enddigraph %}

We have to face it, with LALR we build an automaton for each rule, and try to reuse that rule independent of the context in which it is used. That's keeps our process simple, our automaton small, but it also causes us to lose exactly the information we need to resolve the reduce-reduce conflict in box 3 above :(

So let's look at LR(k) automata/parsers, which use their states to summarise the entire left context of the input. We're basically going to distinguish almost every occurrence of a sort in the grammar, like we did when we made our LL(2) grammar strong. But remember, only if the left context is different, otherwise it doesn't matter.

How do we do this? We duplicate each rule for each terminal in the LL follow set of the sort of that rule. We annotate each of those rules with that terminal. Now we do our usual thing: rule to automaton, epsilons, NFA-to-DFA. But when wiring the epsilons, extra terminal annotations should now match up with the _LALR_ follow set of the occurrence of the sort.

For this particular example, this has the effect of splitting up the above box 3 into two, allowing us to distinguish the context in which `E` and `F` are actually required. In general though, duplicating each rule for each terminal in the LL follow set leads to a very large amount of rules, and plenty of the time this isn't necessary to redundant states in the automaton that do basically the same thing and would have been merged in LALR without any reduce-reduce conflicts.

### Parse table construction algorithm

You've already seen parse table construction by automaton for both LL and the many flavours of LR now. And you've seen parse table construction by _First_ and _Follow_ set for LL. Parse table construction for LR will of course also require _First_ and _Follow_ sets, sometimes including more accurate _Follow_ sets for particular occurrences of sorts. It's mostly NFA-to-DFA (powerset construction) though. 

Where this really gets interesting with _minimal_ LR(1) algorithms and how they create LALR(1) tables when possible, and slightly larger tables when necessary. But that's quite something to figure out, and I haven't gotten to what I wanted to write about most yet, so it will have to wait until the next blog post.

## Recursive Ascent

We finally get to the original impetus for this blog post: recursive ascent parsing. As you might be able to guess, this is the LR analogue to recursive _descent_ for LL. So we're going to generate code that directly executes the LR automaton instead of simulating it by parse table interpretation.

In recursive descent parsing we saw that rules and sorts become functions. Rules call sort functions to parse a sort, and sorts check the look-ahead to choose a rule by which to parse the alternative of the sort.

In recursive _ascent_ parsing we will turn states of the LR automaton into functions. Each function will shift or reduce based on the input and call the corresponding state for that edge. Let's expand our LR(1) example a little bit, and then take a look at the recursive ascent parsing:

:- | :-
{%latex%} S = a E c         {%endlatex%} | {%latex%} \text{(1)} {%endlatex%}
{%latex%} S = a F d         {%endlatex%} | {%latex%} \text{(2)} {%endlatex%}
{%latex%} S = b F c         {%endlatex%} | {%latex%} \text{(3)} {%endlatex%}
{%latex%} E = e e           {%endlatex%} | {%latex%} \text{(4)} {%endlatex%}
{%latex%} F = e e           {%endlatex%} | {%latex%} \text{(5)} {%endlatex%}
{%latex%} S = b E d         {%endlatex%} | {%latex%} \text{(6)} {%endlatex%}
{%latex%} S = b e e a       {%endlatex%} | {%latex%} \text{(7)} {%endlatex%}

The reason I'm adding extra `e`s in rules 3 and 4 is to show that the split up from LR(1) can make the automaton quite big. We'll now have 4 states instead of 2 + an LALR reduce-reduce conflict. The reason I'm adding rule 7 is so we have a state where we might shift or reduce depending on the lookahead, which influences the code we generate. Let's check out the automaton first:

{% digraph A LR(0) automaton for the above grammar %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;

subgraph {
rank=same;
Box0 [shape=none, label=<<table cellborder="0" port="t">
	<tr><td>Box0</td></tr>
	<tr><td>S₁₀</td></tr><tr><td bgcolor="#ddd">S₂₀</td></tr>
	<tr><td>S₃₀</td></tr>
	<tr><td bgcolor="#ddd">S₆₀</td></tr>
	<tr><td>S₇₀</td></tr>
</table>>];
fin [shape=doublecircle, width=0.4, label=""];
Box0:t:s -> fin:n [xlabel="S  "];
}

start1 [shape=none, label="", width=0];
fin [shape=doublecircle, width=0.4, label=""];
S₁₂;
S₁;
S₂₂ [style=filled, fillcolor="#ddd"];
S₂ [style=filled, fillcolor="#ddd"];
S₃₂;
S₃;
S₆₂ [style=filled, fillcolor="#ddd"];
S₆ [style=filled, fillcolor="#ddd"];
Box1 [shape=none, label=<<table cellborder="0" port="t">
	<tr><td>Box1</td></tr>
	<tr><td>S₁₁</td></tr>
	<tr><td bgcolor="#ddd">S₂₁</td></tr>
	<tr><td bgcolor="#aaa">E₄₀</td></tr>
	<tr><td bgcolor="#777"><font color="#fefefe">F₅₀</font></td></tr>
</table>>];
Box2 [shape=none, label=<<table cellborder="0" port="t">
	<tr><td>Box2</td></tr>
	<tr><td>S₃₁</td></tr>
	<tr><td bgcolor="#ddd">S₆₁</td></tr>
	<tr><td>S₇₁</td></tr>
	<tr><td bgcolor="#aaa">E₄₀</td></tr>
	<tr><td bgcolor="#777"><font color="#fefefe">F₅₀</font></td></tr>
</table>>];
Box3 [shape=none, label=<<table cellborder="0" port="t">
	<tr><td>Box3</td></tr>
	<tr><td bgcolor="#aaa">E₄₁</td></tr>
	<tr><td bgcolor="#777"><font color="#fefefe">F₅₁</font></td></tr>
</table>>];
Box4 [shape=none, label=<<table cellborder="0" port="t">
	<tr><td>Box4</td></tr>
	<tr><td>S₇₂</td></tr>
	<tr><td bgcolor="#aaa">E₄₁</td></tr>
	<tr><td bgcolor="#777"><font color="#fefefe">F₅₁</font></td></tr>
</table>>];
Box5 [shape=none, label=<<table cellborder="0" port="t">
	<tr><td>Box5</td></tr>
	<tr><td bgcolor="#aaa">E₄ (c)</td></tr>
	<tr><td bgcolor="#777"><font color="#fefefe">F₅ (d)</font></td></tr>
</table>>];
Box6 [shape=none, label=<<table cellborder="0" port="t">
	<tr><td>Box6</td></tr>
	<tr><td>S₇₃</td></tr>
	<tr><td bgcolor="#aaa">E₄ (d)</td></tr>
	<tr><td bgcolor="#777"><font color="#fefefe">F₅ (c)</font></td></tr>
</table>>];
start1 -> Box0:t;
Box0:t -> Box1:t [label="a"];
Box0:t -> Box2:t [label="b"];
Box1:t -> Box3:t [label="e"];
Box1:t -> S₁₂ [label="E"];
Box1:t -> S₂₂ [label="F"];
Box3:t -> Box5:t [label="e"];
S₁₂ -> S₁ [label="c"];
S₂₂ -> S₂ [label="d"];
Box2:t -> Box4:t [label="e"];
Box2:t -> S₃₂ [label="F"];
S₃₂ -> S₃ [label="c"];
Box2:t -> S₆₂ [label="E"];
S₆₂ -> S₆ [label="d"];
Box4:t -> Box6:t [label="e"];
Box6:t -> S₇ [label="a"];
{% enddigraph %}

Perhaps making both changes at the same time makes this a bad example to show off why LR(1) automata can get so large... If you imagine the automaton without rule 7 you'll see that boxes 3 and 4 are the same except for their ingoing and outgoing edges. This is what happens with longer rules and having to distinguish the final state of the rules for a different look-ahead (boxes 5 and 6 here).

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

This bit of code should give you an idea of the pattern when you have states in the automaton that very predictably result in a sort that we expect to do a goto action on.

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

If you want to keep a recursive ascent code generator simpler you can of course always return a pair. You could also generate the code in [_continuation passing style_](https://en.wikipedia.org/wiki/Continuation-passing_style), where you pass a function that takes the sort and does the goto action instead of accepting a pair as a result. But because the Rust compiler is not very good at tail call optimisation, so I'm not doing that pattern in Rust code here.

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

Note how we're now calling the decrement helper function after the call to `box6` to count one `return` we're going to do immediately after. 

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

The number of returns to do is equal to the size of the body of the rule we are reducing. Of course we immediately decrement because we are going to immediately return. Hence the `map(decr)`, which I didn't inline and do already so keep the constants in the code more intuitive.

```rust
/// S = b e e a .
fn s7(input: &mut Iter) -> Result<(usize, Sort), String> {
    Ok((4, Sort::S)).map(decr)
}

fn lex(input: String) -> Vec<Terminal> {
    input.chars().collect()
}

pub(crate) fn main() -> Result<(), String> {
    let input = env::args().next().expect("Argument string to parse");
    let input = lex(input);
    let mut input = input.iter().peekable();
    box0(&mut input)
}
```

In our main function we can call `box0` with the input. Since this is LR(1) we only need a peekable iterator, that can look ahead 1 terminal.

### Table size = Code size

With both recursive descent and recursive ascent parsing, we're representing the parsing logic directly in code, not as an explicit data representation of a parse table. As such, if you have a larger parse table, you get more code. In LL these parse tables don't grow so quickly, it's mostly related to the size of the grammar, perhaps a bit more if you have larger look-ahead and make the grammar strong. But in LR, when LALR doesn't suffice, parse tables can potentially grow quite large, as we saw to a limited extent with the last example. 

## Recursive Ascent-Descent Parsing

Have you noticed that in the recursive ascent code there are some pretty boring and tiny looking functions? I'm talking about `s12`, `s1`, `s22`, `s2`, `s32`, `s3`, `s62`, `s6`. These will likely be targeted by the inliner of the Rust compiler, so optimise out some function calls. But aren't they a bit much to even generate?

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

Note that in box 6 we now count the number of items in the body of the rule before the dot to come up with the number of returns.

### Left Corners?

The left corner of a rule in the grammar is the left-most symbol in the body of the rule, plus the left corners of any sorts in left corner. So it's basically a _First_ set with the sorts included. I found this is some of the older literature, and figured I'd add a note for myself in here.

There is/was such a thing as left-corner parsing, seemingly mostly used in natural language processing (NLP). NLP mostly uses _ambiguous_ context-free grammars, and typically uses (used?) a backtracking parser to deal with that. These can be slow of course, so optimising it a bit does help. And it turns out left corners helped with this, by adding some filtering that allows the parser to backtrack less. 

# Fin

I really need to stop working on this blog post and publish it already. It's been over a year since I started working on it (on and off, during holidays when I had enough focus time)[^graphviz]. I already had an idea of where to go to next (generalised parsers), but now I also want to study minimal LR(1) automaton/parse table algorithms, and look at continuation passing style again because I think you can pass the left-context as a function argument. This would give you an LALR automaton structure with LR parsing power. Is that a good idea? Don't know, needs testing (or reading papers/blog posts, probably someone else already tried this before).

I usually have a pithy remark or sneak the Kaomoji into the footnotes, but I must be out of practice, because I can't think of a good way to do that...

Ehh, whatever ¯\\\_(ツ)\_/¯

# Footnotes

[^LLdef]: I'm fairly sure my prose description there is the same as a formal definition, and it feel a bit nicer to think about than the ones you can find on [Wikipedia](https://en.wikipedia.org/wiki/LL_grammar#Formal_definition).

[^table]: Technically you'd need to see {%latex%} A₁ {%endlatex%} and {%latex%} A₂ {%endlatex%} as separate symbols and duplicate the rules for {%latex%} A {%endlatex%}, resulting in a larger grammar in correspondence with the larger table. But I couldn't be bothered, and the parse table as shown works just as well. This is relevant to the code size of a recursive descent parser too, since you can just reuse the code for rules 2 and 3 instead of having duplicate code for the two extra rules. What's a recursive descent parser? That comes just a little later in the post, so keep reading ;)

[^LLR]: While I find the [Wikipedia article on LLR](https://en.wikipedia.org/wiki/LL_grammar#Regular_case) confusing, and it makes a good case for why it's not really used, I'm still somewhat intrigued. This is one of those things that will stay on my reading list for a while I think, something I still want to understand further...

[^indirect-recursion]: Indirect left recursion is even worse in LL, because the direct version can still be dealt with by an automatic grammar rewrite algorithm. That's more or less what the node-reparenting trick mentioned at the end of the LL section does. Similarly, there are automatic grammar rewrites for direct right-recursion for LR, and indirect right recursion is more problematic...

[^graphviz]: I hope you appreciate how much time it took to find example grammars to steal (or occasionally develop myself) and especially how much time it took to get GraphViz to output somewhat decent automata of those examples!
