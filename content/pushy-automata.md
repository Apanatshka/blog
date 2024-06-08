+++
title = "Pushy Automata"
date = "2016-05-15"
aliases = ["compsci/2016/05/15/pushy-automata"]
taxonomies.tags = ["theory of computation", "automata", "pda", "context-free language", "context-free grammar"]
+++

Welcome back! This is my second post in a [series](@/theory-of-computation.md) on Automata. I decided to do another theory post first on context-free languages, and only afterwards start on a more implementation-heavy post about implementing this kind of theory in Rust for practically useful stuff. There is of course still code in this post as well :)

I'll start with a quick refresher, but for more details read the [first post](@/finite-automata.md). 

# Finite Automata refresher

Last time, we looked at deterministic and non-deterministic finite automata (DFAs and NFAs resp.), which can handle regular expressions (in fact, they are equivalent). These automata are finite state machines that only (1) take input and (2) return a binary accept/reject output. Finite automata *accept* when they end up in an accept state at the end of the input. You formally describe a finite automaton is by defining:

1. the allowable input symbols (or *alphabet* $\Sigma$), 
2. the *states* $Q$, 
3. the *state transitions* (as a finite mapping $\delta$), 
4. the *start state* $q_0$ and
5. the *final* or *accept states* $F$.

DFAs require every state to have one and only one transition per input symbol. NFAs can have states that don't handle certain input symbols and states that have multiple transitions for the same input symbol. An NFA-$\varepsilon$ even extends the alphabet with the empty string $\varepsilon$, so a transition doesn't have to consume input. The difference between NFAs and DFAs is not in ability (NFA execution is similar to translation to DFA + execution), but NFAs are easier to define. 

# Pushdown Automata

The non-regular language example from the previous post was: "Words in this language start with zeroes and after the zeroes are an equal number of ones". We can't count an arbitrary amount of zeroes in a finite number of states, so we can't remember how many ones we need to see at the end. Therefore, we extend our automata with a *stack*. This stack makes an automaton equivalent in power to the context-free grammar. This type of grammar is used a lot in the reference manuals of programming languages. In fact, there are many tools that allow you to write a context-free grammar and generate a parser from it!  

The finite automaton that has a stack is called a pushdown automaton (PDA). You can think of the stack as a tray dispenser that you can *push* new trays *down* on. Let's kick things off with an example PDA that recognises our non-regular language example:

{{ digraph(gz_file="pushy-automata/non-regular-language.gv", alt="Non-regular language example") }}

So what's happening here? The transitions now have a lot more than than just the input symbol being consumed. After the comma is the top stack symbol that's popped, and after the arrow is the new stack symbol to be pushed. If you look at $q_1$, it is taking 0's off the input and pushed them onto the stack. Then it takes in as many 1's as 0's, by popping a 0 off the stack for every $1$ in the input.  
The outer states are only there to make sure we have a fully empty stack before we go into a final state. The \$ is usually used as an *End Of Stack* character. You can also change the definition of the PDA to already hold 1 character on the stack at the start. This is part of the definition as you find it on [Wikipedia](https://en.wikipedia.org/wiki/Pushdown_automaton#Formal_definition). 

## Determinism

The previous PDA was non-deterministic, but we can make it deterministic. I've left off the stuck state, but there should be no overlapping transitions and all transitions consume either input, stack or both. 

{{ digraph(gz_file="pushy-automata/non-regular-deterministic.gv", alt="Non-regular language example, deterministic") }}

Now in general, we *cannot* change our PDAs to a deterministic version (deterministic PDAs are strictly less powerful). For example, take the language of even-length binary palindromes. This language can be recognised by the following non-deterministic PDA, but not by a deterministic one:

{{ digraph(gz_file="pushy-automata/even-binary-palindromes.gv", alt="Even-length binary palindromes") }}

If you think about it, it makes sense that a non-deterministic PDA is more powerful than a deterministic PDA. With the NFAs in the previous post we had just a finite amount of states we could be in while executing, and you can model that with (an exponential amount of) states in a DFA. But for a PDA, it isn't just the state that matters, but the stack as well. Since the stack isn't finite, we can't just model it in more states. 

## Code

These PDAs are a bit annoying to write as is. Epsilons for the input character mean that we're not advancing the input. We could write a direct encoding of the formal definition, but then we need to resolve epsilons at runtime. The execution would become pull-based, asking for input and the top of the stack when we need it. Somehow that feels wrong to me.  
So instead we're going to adapt our definition of PDAs, so that we can write code that's still driven by the input. Let's see if we can eliminate epsilons in the input position. There are five cases:

- Add something to the stack at the start
  - Example: $q_0 \rightarrow q_1$
  - Fix: Allow stack to start with a static bunch of symbols on it
- Add multiple things to the stack on a certain input
  - Fix: Allow pushing multiple things on the stack
- Express some non-determinism easily by doing nothing with the stack or the input
  - Example: $q_1 \rightarrow q_2$
  - Fix: This is the tradition NFA epsilon move, we can do a local powerset construction for this
- Remove multiple things from the stack on a certain input
  - Fix: Allow popping multiple things off the stack
- Remove stuff from the stack at the end of the input
  - Example: $q_1 \rightarrow q_2$
  - Fix? Not sure if there is one, so we'll just have to deal with this one

With the above fixes, we can get a PDA that will always either advance one step in the input, or at the end of the input advance on the stack. These fixes can be expressed in the original definition, so it still has the same power. Here's the new version of the PDA:

{{ digraph(gz_file="pushy-automata/even-binary-palindromes-v2.gv", alt="Even-length binary palindromes, v2") }}

So with that, we can go to the code. I apologise for the messier transition functions. Those don't correspond to the diagram as clearly. In retrospect this approach to the transition functions would have also worked for the other PDA, although it would be slightly less efficient (I think). 

{{ rust(rust_file="pushy-automata/binary_palindrome/src/main.rs") }}

I left in extra prints to observe the behaviour of the PDA:

```rust
[(0, [2])]
[(1, [2, 0])]
[(1, [2, 0, 0]), (2, [2])]
[(1, [2, 0, 0, 1])]
[(1, [2, 0, 0, 1, 0])]
[(1, [2, 0, 0, 1, 0, 1])]
[(1, [2, 0, 0, 1, 0, 1, 0])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1]), (2, [2, 0, 0, 1, 0, 1, 0])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1]), (2, [2, 0, 0, 1, 0, 1, 0, 1])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1]), (2, [2, 0, 0, 1, 0, 1, 0, 1, 1]), (2, [2, 0, 0, 1, 0, 1, 0])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0]), (2, [2, 0, 0, 1, 0, 1])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1]), (2, [2, 0, 0, 1, 0])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0]), (2, [2, 0, 0, 1])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1]), (2, [2, 0, 0])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0]), (2, [2, 0])]
[(1, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0]), (2, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1]), (2, [2])]
[(3, [])]
The input is accepted
```

Something to note is that the amount of `(state, stack)` tuples peaks at 3 and is mostly 2. That's not so bad. But in general you can have input strings with much worse behaviour! ([Try](https://github.com/Apanatshka/blog/tree/zola/content/pushy-automata/binary_palindrome/) a long string with only zeroes for example). 

# Context-free grammars

A context-free grammar (CFG) has consists of rules which are sometimes called production rules or substitution rules. Those names are basically two ways to look at the grammar: as a way to produce 'sentences' of the language that the grammar describes, or to reduce input to check if it's part of the language.  
These rules are written with terminals (symbols from the alphabet), and sorts (or grammar variables or non-terminals). A sort is defined by one or more rules. Depending on the grammar formalism, you may see $\leftarrow$, $\rightarrow$, $=$ or $::=$ between the sort and the body of the rule. Let's look at an example:

| | |
| :- | :- |
| $S = 0 S 0$       | (Rule-0) |
| $S = 1 S 1$       | (Rule-1) |
| $S = \varepsilon$ | (Rule-$\varepsilon$) |

This CFG describes the even-length binary palindromes that our last PDA also described. It has a single sort $S$, the *start sort* of the CFG. The zero and one are terminals, symbols from the alphabet. The epsilon is still the empty string. I've labelled the rules so I can refer to them later. 

To recognise a string, we start with the input and try to reduce part of it to a sort according to one of the rules. We keep substituting until we having only a single start sort left. This is called a *derivation*. For example:

| | |
| :- | :- |
$0010101111010100$             |
$00101011\varepsilon11010100$  | ($\varepsilon$ insertion)
$00101011S11010100$            | (Rule-$\varepsilon$)
$0010101S1010100$              | (Rule-1)
$001010S010100$                | (Rule-1)
$00101S10100$                  | (Rule-0)
$0010S0100$                    | (Rule-1)
$001S100$                      | (Rule-0)
$00S00$                        | (Rule-1)
$0S0$                          | (Rule-0)
$S$                            | (Rule-0)

Of course the other way around also works, start with the start sort and expand sorts non-deterministically to end up with the string to recognise. 

## Translation to PDA

Now CFGs are equally powerful as the PDA. That means that similar to regular expressions and NFAs/DFAs, we can translate from one to the other. Let's do the grammar to automaton, since you're more likely to write a grammar that you want to execute than the other way around. The idea is that you have a PDA with an EOS symbol and the start sort. Then you get to the 'central' state in the PDA. This state replaces the topmost sort on the stack with the *reversed* body of one of it's rules (non-deterministically of course). If the topmost thing on the stack is a terminal instead, it will match the input against the terminal and drop both. Because the rule body was pushed on the stack in reverse that works out. When the EOS symbol is found and the input is found we go to the accept state. 

Let's look at the PDA for the binary palindrome grammar:

{{ digraph(gz_file="pushy-automata/even-binary-palindromes-grammar.gv", alt="Even-length binary palindromes, translated from the grammar") }}

I went for the PDA which starts with an initialised stack and can manipulate multiple things on (the top of) the stack at once. That gives a more compact PDA, and is also closer to an implementable state. Sadly this example doesn't visibly show that the bodies of the rules are reversed, because all rules in this grammar are symmetrical. 

It's interesting to see that this PDA is actually smaller in states than our hand-written one. But this one does have some more overhead because it's pushing a lot of stuff on the stack including sorts. Let's see if we can reduce that overhead a little by at least making the transitions that don't consume any input into transitions that do. For that we need to merge a rule like $\varepsilon, S \rightarrow 0 S 0$ with other rules that will come afterwards which do consume input. That's $0, 0 \rightarrow \varepsilon$ in this case. So combining the two rules gets us $0, S \rightarrow 0 S$, by adding the input symbol consumption and resolving the stack pop. We can do the same with the other transition that takes no input. The last rule to resolve is $\epsilon, S \rightarrow \varepsilon$. This one can be merged with $\epsilon, S \rightarrow 0 S 0$ and $0, 0 \rightarrow \varepsilon$ to form $0, S \rightarrow 0$ and with the other two transitions to form $1, S \rightarrow 1$. 

{{ digraph(gz_file="pushy-automata/even-binary-palindromes-merged.gv", alt="Even-length binary palindromes, merged grammar rules") }}

Now the sort $S$ has been changed from a fairly useless overhead to a marker of "we're not halfway yet". In our hand-written PDA this was not a symbol on the stack but a different state. 

When you [implement](https://github.com/Apanatshka/blog/tree/zola/content/pushy-automata/binary_palindrome/src/grammar_based.rs) this PDA you get an output that shows that there is one redundant state that it's always in:

```rust
[(0, [2, 3]), (1, [])]
[(0, [2, 0, 3]), (0, [2, 0])]
[(0, [2, 0, 0, 3]), (0, [2, 0, 0]), (0, [2])]
[(0, [2, 0, 0, 1, 3]), (0, [2, 0, 0, 1])]
[(0, [2, 0, 0, 1, 0, 3]), (0, [2, 0, 0, 1, 0])]
[(0, [2, 0, 0, 1, 0, 1, 3]), (0, [2, 0, 0, 1, 0, 1])]
[(0, [2, 0, 0, 1, 0, 1, 0, 3]), (0, [2, 0, 0, 1, 0, 1, 0])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1]), (0, [2, 0, 0, 1, 0, 1, 0])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1]), (0, [2, 0, 0, 1, 0, 1, 0, 1])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1]), (0, [2, 0, 0, 1, 0, 1, 0])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0]), (0, [2, 0, 0, 1, 0, 1])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1]), (0, [2, 0, 0, 1, 0])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0]), (0, [2, 0, 0, 1])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1]), (0, [2, 0, 0])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0]), (0, [2, 0])]
[(0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0, 3]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 0]), (0, [2, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1]), (0, [2])]
[(1, [])]
The input is accepted
```

This redundant state comes from the two rules that don't re-add the $S$. These rules basically try to predict at every point in the input that this was the last input symbol of the first half, which most of the time isn't going to be true. We could change them to instead predict that this was first input symbol of the second half, which can only happen when the second value on top of the stack is the same as this input symbol: $0, 0 S \rightarrow \varepsilon$.  
This rule without the overhead is just a simple combination of the old rules $\varepsilon, S \rightarrow \varepsilon$ and $0, 0 \rightarrow \varepsilon$. It's only because we combined with the third rule $\varepsilon, S \rightarrow 0 S 0$ that we ended up in a sub-optimal situation.[^optimality] At this point it's pretty clear that instead of push and popping two things of which the second is the $S$, can also be expressed as just another state. 

We're going to skip translating PDAs to CFGs, as that's a less interesting thing to do in my opinion. It shows that PDAs aren't more powerful than CFGs, but isn't used for something practical as far as I know. So---at least for me---it's enough to know that someone else has proven this property. 

## Ambiguity

The binary palindrome example has some non-determinism in there that you can't get rid of, but in the end it still has only one way to check/derive a word in the language. When you can apply multiple rules in multiple orders and still find the same word, you get into the issue of ambiguity.
 
Now in general you can have multiple sorts while in the middle of a derivation. In that case you can always pick a different order in which to substitute the sort for one of its rules and therefore change the way you derive a word. So that's not a very useful definition of ambiguity. To ignore this part of the order of derivation, we'll just arbitrarily pick an order in which you should to substitute sorts in a derivation: left-to-right. This gives you a so-called leftmost derivation. If there are still multiple left-most derivations, your CFG is ambiguous. 

Let's look a simple ambiguous grammar:

| | |
:- | :-
$\text{Expr} = \text{Expr} + \text{Expr}$  | (Addition)
$\text{Expr} = \text{Expr} * \text{Expr}$  | (Multiplication)
$\text{Expr} = 0$            | (Zero)
$\text{Expr} = 1$            | (One)

This is a basic arithmetic expressions grammar. And yet when you write multiple additions or multiplications, you get different possible derivation trees:

| | |
:- | :-
{{ digraph(gz_file="pushy-automata/arith-derivation-tree1.gv", alt="arithmetic expressions derivation tree 1") }} | {{ digraph(gz_file="pushy-automata/arith-derivation-tree2.gv", alt="arithmetic expressions derivation tree 2") }}

These trees show how the derivations went from sorts to terminals. In a way, they also show an ordering, where the left one does the multiplication first and the right one does the addition first. Although this is an ambiguous grammar, it doesn't have to be. The language that it captures, arithmetic expressions, has a notion of ordering between addition and multiplication, namely that multiplication goes first. This is called precedence: multiplication takes precedence over (binds tighter than) addition. For this unambiguous language you can explicitly encode the precedence rules in the grammar to get an unambiguous grammar. 

### Inherently ambiguous

There are actually Context-Free Languages (CFLs) that are inherently ambiguous, they can only be captured by ambiguous CFGs. Here's an example of an ambiguous grammar that captures an inherently ambiguous language:

| | | | | | | |
:- | :- | :- | - | :- | :- | :-
$S$   | $= A_b C$       | (Equal-A-B)         | &nbsp; | $S$   | $= A B_c$       | (Equal-B-C)
$A_b$ | $= a A_b b$     | (A-B)               | &nbsp; | $B_c$ | $= b B_c c$     | (B-C)
$A_b$ | $= \varepsilon$ | (A-B-$\varepsilon$) | &nbsp; | $B_c$ | $= \varepsilon$ | (B-C-$\varepsilon$)
$C$   | $= c C$         | (C)                 | &nbsp; | $A$   | $= a A$         | (A)
$C$   | $= \varepsilon$ | (C-$\varepsilon$)   | &nbsp; | $A$   | $= \varepsilon$ | (A-$\varepsilon$)

This describes a language that has either (1) a number of $a$'s followed by an equal number of $b$'s followed by an arbitrary number of $c$'s, or (2) an arbitrary number of $a$'s followed by a number of $b$'s followed by an equal number of $c$'s. These two options overlap when you have an equal number of $a$'s, $b$'s and $c$'s, which results in an inherent ambiguity in this case. 

## Pumping lemma

In the [previous blog post](@/finite-automata.md) I originally skipped the description of the pumping lemma for regular languages. But after some feedback on the post, I [added the description of the basic idea](@/finite-automata.md#addendum). The idea is that any regular language (although also other languages) will have the property of a pumping length, where any word in the language larger than this length can be pumped up to a larger word that's still in the language. For a language with a finite number of words the pumping length is larger than the largest word in the language. For infinite languages you cannot do this, which means that there are words in the language where you can find a part of the word that you're allowed to repeat an arbitrary amount of times. This arbitrary repetition corresponds with a loop in the DFA or NFA that describes the language. 

The pumping lemma for context-free languages is similar to that of regular languages. We have a pumping length and can split words larger than the pumping length into parts. Instead three parts of which the middle can be repeated, in CFLs we split words into five parts. The second and fourth part can be repeated an arbitrary amount of times as long as they are both repeated the same number of times. This makes sense because as we've seen, we can remember a bunch of things with the stack in a PDA so we can keep two parts of a word in sync with respect to repetition. From a CFG perspective it also makes sense, because the repeated parts are basically the two terminal parts that surround a recursively defined variable (for example). 

If you want to look into this further you can look up the [wikipedia page](https://en.wikipedia.org/wiki/Pumping_lemma_for_context-free_languages), or another online resource I don't know of, or a CS book on this topic. I used "Introduction to the Theory of Computation" by Michael Sipser, which goes into detail about how to write proofs with the pumping lemma (and many other interesting things). 


[^optimality]: Though I can't guarantee that stuff will be optimal in general with this approach. I guess the approach is pretty vaguely defined anyway. Ehhh.. whatever ¯\\\_(ツ)\_/¯
