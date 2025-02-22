+++
title = "Finite Automata"
date = "2016-04-10"
updated = "2016-04-17"
aliases = ["compsci/2016/04/10/finite-automata"]
taxonomies.tags = ["theory of computation", "automata", "nfa", "dfa", "regular expression", "rust"]
+++

What do Turing machines and regular expressions have in common? One is a theoretical model of a computer, and can be used to prove that some things cannot be computed. The other is a practical tool for matching strings. And yet they are both based on a simple *computational model*: a (very constrained) finite state machine (FSM). 

In this blog post we'll go over the basics of this type of FSM and instead of going over proofs, we'll go over examples and little implementations in Rust. For more information about this blog post series, see [this announcement post](@/theory-of-computation.md). 

# An exercise in minimalism

We'll start with the simplest, most restricted version of our FSMs. These are great for proofs because everything explicitly defined and super simple. But they are not so great to construct by hand, so we'll discuss a nicer version afterwards and relate *that one* to regular expressions. Turing machine will not be part of this post. 

## Deterministic Finite Automaton (DFA)

DFAs are FSMs (*automata*) that work on a *finite* input and give a boolean output. *True* means the input was recognised as part of the 'language' that the DFA encodes, *false* means it is not part of the language. *Deterministic* automata define all their (*finite* amount of) states and *exactly one* transition for every possible pair of state and input.  
The way you formally describe a DFA is by defining:

1. the allowable input symbols (or *alphabet* $\Sigma$), 
2. the *states* $Q$, 
3. the *state transitions* (as a finite mapping $\delta$), 
4. the *start state* $q_0$ and
5. the *final* or *accept states* $F$.

### Example: Binary string

Let's construct a DFA that can recognise inputs that start with a one, has at least two zeroes after that, and then at least one more one, after which the 'word' ends. 

{{ digraph(gz_file="binary-string-dfa.gv", alt="Binary string DFA") }}

Note that I've already started cheating with the construction of this DFA. Not every state handles all symbols in the alphabet (0 and 1). This partially defined DFA is usually easier to write and read. The usual way to make it fully defined is to add an explicit *stuck state*. All the unhandled symbols go to that state, and with any next input the DFA will stay in that state. 

### Code code code

In Rust we can do the partial definition of the DFA with an `Option` type:

{{ rust(rust_file="binary_string/src/main.rs") }}

(The crate is in this [blog's repository](https://github.com/Apanatshka/blog/tree/zola/contents/finite-automata/binary_string/))  
So `None` is the stuck state and the 'real' states are wrapped in a `Some`. In this code the transitions are given as a function, not a mapping. When you generalise this into an automaton library (there [are](https://crates.io/search?q=automaton) [several](https://crates.io/search?q=automata) on [crates.io](https://crates.io/)), you're more likely to end up with a map. 

### Memory

Note that DFAs are so restricted that they don't really have mutable memory. Any kind of memory of what you've already seen of the input needs to be statically encoded in the states of the state machine. This can get a little awkward when you want to recognise binary strings that have a 1 as the second to last symbol:

{{ digraph(gz_file="binary-string-dfa-memory.gv", alt="Binary string DFA encoding memory") }}

We remember the last three input symbols in our states. That gives us $2^3$ states, an exponential relation. So with these kinds of problems, you really don't want to design these DFAs by hand. 

## Non-deterministic Finite Automaton (NFA)

Non-determinism allows states to have multiple transitions per symbol. That means that when you simulate an NFA, you can be in multiple states at once. This allows us to avoid the exponential blowup of the last example:

{{ digraph(gz_file="binary-string-nfa.gv", alt="Binary string NFA") }}

Although this NFA is easier to describe, it's still always translatable to a DFA. This translation algorithm is called powerset construction or subset construction. The powerset of a set is the set of all combinations: $\mathbb{P}(\{0,1\}) = \{\emptyset, \{0\}, \{1\}, \{0,1\}\}$

### Powerset construction

1. The *alphabet* stays the same.
2. We use the powerset of the states of the NFA to create the states for the DFA. (That's where the exponential blowup comes from, **if** all those states are used.) The state with the $\emptyset$ is the *stuck state* that we already saw earlier. 
3. The transitions are based on the simulation of the NFA. So if you are in $\{q_0, q_1, q_3\}$ then the transition with symbol $\sigma_2$ takes you to the state in the DFA that is labelled with states of the NFA that are reachable with $\sigma_2$ from the states $q_0$, $q_1$ and $q_3$. 
4. The start state is still the same although now called $\{q_o\}$ instead of $q_o$. 
5. The final states are every state that has an NFA final state in its set. 

### Epsilon moves

The empty string is referred to with the greek letter $\varepsilon$. If you allow transitions in an NFA to be labelled with $\varepsilon$, you get the NFA-$\varepsilon$ type of automata. This changes the powerset construction slightly, because an $\varepsilon$ move from the start state in the NFA $q_0$ means that the start state of the DFA will be one of the compound states $\{q_0, ...\}$. This class of automata is still not more powerful than the NFA or DFA. But it *is* useful for the definition of the basic regular expression operators.

### Regular expressions

Now just to warn you: regular expressions in programming were once based on this automata theory, but have since been made much more powerful. Regex can describe much more than just *regular* languages. 

**The basics** Ok, say you have an empty regular expression. That's an NFA with one state, the start state, which is also a final state. It only recognises $\varepsilon$.  
A regular expression that matches exactly `1` is the same as an NFA with two states, the start state, and a separate final state with a transition in between labelled with `1`. 

**Option** Let's take two regular expressions put an 'or' (`|`) in between. If either one matches, the whole regex matches. If we have two NFA equivalents, we can make a new start state and $\varepsilon$-transitions to the two old start states. 

**Concatenation** Let's take two regular expressions and stick them together, one after the other. In NFA-land that means that the final states of the first NFA become normal states with $\varepsilon$-transitions to the start state of the second NFA. 

**Kleene Star** This is the `*` in a regular expression, a zero-or-more. With this and the option, you can make a one-or-more (`+`). The way this works in NFA-land is as follows: Create a new start state, which is a final state, and give it an $\varepsilon$-transition to the old start state (the zero part). Give the old final states $\varepsilon$-transitions to the old start state (the more part). 

## Limitations and Non-regular languages

Say you want to recognise binary strings that starts with any number of zeroes, but then is followed by an equal number of ones. Because the number of zeroes is unbounded, we cannot use a finite number of states to count how many zeroes we've seen so far, to match with the amount of ones to come. Basically, it's impossible to use DFAs or NFAs to recognise 'words' of this language. 

More generally you can prove languages non-regular with the [pumping lemma for regular languages](https://en.wikipedia.org/wiki/Pumping_lemma_for_regular_languages). I don't have a super-intuitive way of explaining the pumping lemma, so if you want to know more, try reading the theory on it. *(Or do I? See the [Addendum](#addendum)!)*

# P.S.

Next blog post we'll look at context-free languages, which is a superset of the regular languages. They can be described with context-free grammars (which you may know from the definitions of programming language syntax) and which you can recognise with an automaton that has a single stack for memory. 

I was going to write some more Rust code for you to look at, like a generally usable automaton library and the powerset construction. But this post has taken so long, it is already long, and there are Rust crates available with that kind of stuff... So, whatever ¯\\\_(ツ)\_/¯

<hr/>

# Addendum

## Non-regularity and the pumping lemma

Most of the examples I've seen of non-regular languages have some form of unbounded counting in them. But if you have a different language that doesn't seem to fit, you can try to prove it non-regular too. The usual way to prove that a language is not regular, is to show that the [pumping lemma for regular languages](https://en.wikipedia.org/wiki/Pumping_lemma_for_regular_languages) doesn't hold for the language. The pumping lemma is a property that all regular languages have (though there are some non-regular languages which are have this property). The basic idea is this:

1. Your language either consists of a finite set of words, and is therefore regular, or it consists of an infinite set. (The finite set means you can just union the DFAs for every separate word. )
2. With an infinite set of words, and a finite alphabet, you'll have words that have more symbols in them than your language's DFA has states. That means the DFA has to loop somewhere.  
3. Every word in your language that is longer and uses a loop has three parts: the part before the loop, the part in the loop and the part after the loop. Such a word is part of an infinite class of words where you can repeat the middle part however many times you like. 

To prove that a language doesn't have this *pumping length* from the lemma, you need to be pretty abstract and use a cleverly chosen word. So if you want to write such a proof and haven't done it before, I advise you to look up some examples!

*Thanks to [/u/so_you_like_donuts](https://www.reddit.com/user/so_you_like_donuts) for giving an [intuitive description of the pumping lemma](https://www.reddit.com/r/rust/comments/4eq01l/my_first_blog_post_finite_automata_with_a_bit_of/d22bw6q?context=3) which drove me to write this addendum.*

## Next post

I had another week to read other blog posts, including an excellent one about [automata in rust for string indexes](http://blog.burntsushi.net/transducers/) and now I feel bad for not giving you the Rust code for every algorithm I explained. So, I might just postpone the blog post about context-free languages and dive into regex first. I doubt I'll do better than the existing [regex crate](https://crates.io/crates/regex), but, you know, whatever ¯\\\_(ツ)\_/¯
