---
layout:   post
title:    "Optimising CTree and strs"
date:     2018-05-08
category: CompSci
tags:     [Rust, Stratego, interpreter, optimisation]
---

Once upon a time, I wrote an [interpreter for Stratego Core](@/2017-08-06-a-stratego-interpreter-in-rust.md) in Rust, which I named `strs`. Stratego Core is the core language that Stratego is compiled to before the compiler goes further (to Java, or previously to C). A core language is an intermediate representation that is a subset of the surface language. 

While I optimised that interpreter quite a bit, I noticed that the CTree (Stratego Core Abstract Syntax Tree) that the compiler spit out for me to interpret was very unoptimised. Therefore one the plans I described at the end of the blog post was a little tool for Copy Propagation on CTree files. This post is about that tool, and the optimisations in the interpreter that made it obsolete again. 

# Copy Propagation

Copy propagation looks for assignments of one variable to the value of another variable, and inline that assignment. So for a program with `x = y`, copy propagation will eliminate that assignment and instead replace subsequent uses of `x` with `y`. When Stratego is translated to Stratego Core, the syntactic sugar is removed by introducing lots of new variables. Copy propagation is a good clean-up optimisation to remove many superfluous assignments that result from the desugaring. Of course this simple idea is not so simple in Stratego...

## Copy Propagation for Stratego

Here's a really short refresher, look at the [`strs` blog post](@/2017-08-06-a-stratego-interpreter-in-rust.md) if you need more. In Stratego Core we have a current term. There are term variable and strategy variables, new ones can be introduced with scopes and lets respectively. We can match against the current term, and we can build another term in its place, where both use patterns that can have term variables in them. We can call strategies and primitives. There are generic traversals over terms, basic `id` and `fail` strategies, and the guarded choice as a kind of if-then-else.

An assignment in Stratego Core can be found when you match the current term against a pattern. What we're looking for in particular is building a plain variable, then matching another plain variable (no more complicated patterns). However, matching against a variable that already has a bound value is _not_ an assignment. Then the meaning is an equality test. So we can't just look in a sequence of strategies for a `..., Build(Var(...)), Match(Var(...)), ...`, we need to check whether the match is actually a binding of a fresh variable. The simplest way I know to do so is by adapting the interpreter we had, to instead to an "abstract interpretation". 

# Copy prop on CTree using abstract interpretation

Abstract interpretation is a way of using an interpreter for static analysis of a program. The abstract part is usually about the values of variables, where you choosing some more vague, a property, instead of a real value. That way you can analyse the program without needing input or user interaction. Ideally you also figure out a value space or some other way to make sure you can't go into loops, so your analysis actually terminates (and does so within a reasonable amount of time). 

The term abstract interpretation is used in academia for something more sophisticated: given an interpreter, make systematic changes that provably preserve the semantics of the interpreter, while abstracting it. This way you can design an analysis of a program that provably abstracts over the concrete semantics of the program as given by the original interpreter. This is not an easy topic to read about, and until a [fellow PhD student](http://svenkeidel.de/) started working on it, I was convinced that it's also not easy to execute. I wish I could link you to his publication but it's still under review. 

## Values

So let's set up an interpreter that always terminates by not going into strategy calls. The value space, which would also be the value for the current term, should be a "yes-no-maybe" kind of value:

```rust
trait CTreeOptimize {
    type Context;

    fn optimize(self, context: &mut Self::Context) -> Self;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum Value<'s> {
    UnBound,
    MaybeBound,
    Bound,
    BoundTo(InternedString<'s>),
}

impl<'s> Value<'s> {
    fn lub(&self, other: &Self) -> Self {
        use self::Value::*;

        if self == other {
            return self.clone();
        }

        match (self, other) {
            (&BoundTo(_), &BoundTo(_)) |
            (&BoundTo(_), &Bound) |
            (&Bound, &BoundTo(_)) => Bound,
            _ => MaybeBound,
        }
    }
}

impl<'s> PartialOrd<Value<'s>> for Value<'s> {
    // a <= b iff lub(a,b) == b
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Scope<'s> {
    strategy: FnvHashMap<
        InternedString<'s>,
        (::std::result::Result<FnvHashSet<InternedString<'s>>, DynamicCall>,
         usize),
    >,
    term: FnvHashMap<InternedString<'s>, Value<'s>>,
    is_overlay: bool,
    is_unbound_overlay: bool,
}
```

We can define our interpreter using the `CTreeOptimize` trait, which just goes over the tree, since we're not following strategy calls. The `Context` is different depending on if we're in a strategy or outside of it. Inside there's the current term and the stack of scopes, outside the context is `()`. The values of variable can at most record that they're bound to the value of another variable. The partial order is:

```
      MaybeBound
     /          \
UnBound        Bound
              /  |  \
BoundTo("a_1")  ...  BoundTo("z_999")
```

The Least-Upper-Bound (`lub`) operation gives a conservative combination of values when you need to merge values from multiple branches of execution (guarded choice). 

## Match and Build

The context `c` in the next snippet is a tuple of the current term and the scopes stack. So we set `c.0` to change the current term, and do lookups in `c.1` for the term variables. The current term is an option of the name of the variable that was built previously. With `into` it's turned into `Bound` or `BoundTo(_)` depending on whether it's `None` or `Some(_)`. 

```rust
Strategy::Match(mt) => {
    // If we match against a single variable...
    if let MatchTerm::Var(v) = mt {
        // ...and it's guaranteed to be UnBound at this point...
        if c.1.get_term(v) == Some(Value::UnBound) {
            // ...we set the name to be bound to what the current value is
            c.1.set_term(v, (&c.0).into());
            // if the current value is also a known binding, we've found an alias
            //  that we can eliminate
            if c.0.is_some() {
                return Strategy::Id;
            } else {
                // Otherwise the current is now a known binding
                c.0 = Some(v);
            }
        }
    }
    // We optimize match patterns by replacing variables when they are aliases
    Strategy::Match(mt.optimize(c))
}
Strategy::Build(bt) => {
    // We optimize build patterns by replacing variables when they are aliases
    let bt = bt.optimize(c);
    // If we build a single variable...
    if let BuildTerm::Var(v) = bt {
        // ...and the variable is guaranteed to be bound at this point
        match c.1.get_term(v) {
            Some(Value::Bound) => {
                // ...we record that the current term is also this known binding
                c.0 = Some(v);
                return Strategy::Build(bt);
            }
            // Note that we don't need to handle BoundTo because that would have
            //  been an alias that was replaced by the optimize call on the pattern
            _ => {}
        }
    }
    // If the build is not a single variable or a possibly unbound variable, we
    //  don't know for sure if the current term is also known as a variable
    c.0 = None;
    Strategy::Build(bt)
}
```

## Scopes and Sequences

For a Stratego Core scope construct we introduce a scope to the stack of scopes, optimize the strategy in the scope, then pop the scope again. From this scope we can learn which variables were bound to another variable and therefore removed, so we also remove them from the list of fresh variables as they are no longer used.

For sequences we do something more than just optimise the strategies in it. Since optimised single variable matches are turned into `id` strategies, we can just filter all `id` strategies in a sequence to get rid of that. While we're at it, we can also cut the sequence short if we run into a `fail`. Lastly, when we find a sequence of builds, where the first build has all bound variables, therefore it is guaranteed to succeed, then we remove that build, since the second build will immediately override the current term value. These are all patterns that I noticed while scrolling through some CTree files that the compiler spit out. 

# Results of applying `ctree_opt`

In the previous `strs` post I benchmarked against a `benchexpr10` and a `benchexpr20` program. The first takes a very short time, the second a fairly long one. So I created a `benchexpr15`, which was easy enough. The ctree of that program is `62221 B`. The optimised version with `ctree_opt` is `58003 B`, an almost 7% decrease in file size. 

Now, when we run the benchmark, and the optimised version, we get the timings:

```
strs benchexpr15.ctree       3.25s user 0.04s system 99% cpu 3.317 total
strs benchexpr15.opt.ctree   3.13s user 0.06s system 99% cpu 3.223 total
```

Consistently 3% faster with optimisation. 

# Making name lookup cheap

The `ctree_opt` tool really didn't do much more than remove some redundant matches and builds. Given the improvement on the benchmark, that suggests that there might be something wrong with variable lookup. If we look at the interpreter, it has a stack of scopes. Each scope has a hashmap from variable name to value (or `None` if unbound). The interpreter looks up the name by trying each hashmap while going down the stack. 

We could instead keep a hashmap of value stacks, but this would make pushing and popping scopes much more expensive. We could keep around a hashmap of name to offset in the stack of scopes with much the same problem. We could also run a static analysis to find the nearest scope that defines the term variable and save the offset to that scope for each variable. 

If we're speaking of offsets anyway, why not use offsets within a scope too, so we don't need the hashmap lookup anymore? To really go off the rails here, we can fuse all the scopes into one big vector (we know their sizes statically), as long as we dynamically keep track of at which index a scope began. Now we have one vector of values, and a complicated calculation of offsets. You can guess the result: off-by-one errors. And slow interpreter startup, as the offset calculation of all names (term and strategy) are done at startup for the entire program + libraries. 

For the slow startup there was a quick-and-dirty solution: Use strategy names, starting from main, and check strategy reachability to do course-grained dead-code elimination. Most of the standard library isn't used anyway. For the off-by-one errors, only time and patience could fix the problems. In the end I got it working almost completely, at least complete enough to run the benchmark successfully again:

```
strs benchexpr15.ctree       2.36s user 0.05s system 99% cpu 2.423 total
strs benchexpr15.opt.ctree   2.34s user 0.04s system 99% cpu 2.396 total
```

Optimisation seems to have a positive effect still, but now it falls within the run-to-run variance. 

So there you have it. A CTree optimisation tool with abstract interpretation, and a slightly crazy scheme to model the stack memory of the interpreter with a single vector. Not that that's completely working yet, and I haven't had time to fix the last issues for a while now, but such is the fate of my side-projects constantly. So I'll just close with: meh, whatever ¯\\\_(ツ)\_/¯
