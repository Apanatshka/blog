---
layout:   post
title:    "An Optimisation Mystery"
date:     2019-09-30
category: CompSci
tags:     [Rust, Stratego, interpreter, optimisation]
---

This isn't my usual kind of blogpost, in that the mystery in the title will not be solved in post. 

Once upon a time, I wrote an [interpreter for Stratego Core]({% post_url 2017-08-06-a-stratego-interpreter-in-rust %}) in Rust, which I named `strs`. Stratego Core is the core language that Stratego is compiled to before the compiler goes further (to Java, or previously to C). A core language is an intermediate representation that is a subset of the surface language. 

I also wrote [a follow up post about an optimisation tool for Stratego Core]({% post_url 2018-05-08-optimising-stratego-core %}). That tool does a kind of abstract interpretation, analysing the code statically and removing redundant local variables that ended up in the code because of how the Stratego compiler desugars the surface language into the core language. The blog post ends with a brief description of another general optimisation in the preprocessor of my interpreter. That optimisation was makes looking up local variables much cheaper. This kind of killed the use-case of the optimisation tool, but that's ok. 

I'd like to report that the general optimisation of name lookup is now fully functional. In the last blog post it was working well enough to run a particular benchmark, but still failed some tests. I eventually found the bug, and fixed it. 

In this post I'd like to explain the name lookup optimisation a bit better. Then we'll continue with the optimisation mystery: improving the startup time of the interpreter, which had a detrimental effect on the interpreter itself, despite the fact that I barely touched the code relevant the interpretation of a Stratego program.

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

# Improving start-up time

The optimisation mystery is based on this idea that we may be able to speed up the preprocessing in the interpreter by building a call-graph early on, just after reading in the definitions. Then we use the call-graph from the `main` strategy and find all reachable definitions. Only those definitions get preprocessed further with the offset-based names. 

It should be simple enough then to take the earliest step in which the program is read, and, through a bottom-up traversal, collect the calls to other strategies for each definition. However, it grated that in order to improve the start-up time of the interpreter, in order to speed up the preprocessing, I'd be adding another tree traversal that had a complexity in the size of the program first. That traversal would ensure another more complicated one would not have to go over the entire program, but still...

And so, I stupidly decided to fold this new traversal into an already existing one. Manual "deforestation"[^deforestation]. Might be a good idea for performance, but not for testing separate ideas separately. Also may or may not result in harder to read code. (That depends a bit on how much each traversal actually _does_. 

Ok, so we add to an existing traversal. Let's have a look at the traversals then: 

1. There's the `aterm` crate I wrote, which parses the text in a `.ctree` file. 
2. Then the `preprocessing/ctree` module defines enums and structs [following the CTree grammar](https://github.com/metaborg/strategoxt/blob/4359e26c902ad9f0374911b36517161f2e9c0b5b/strategoxt/syntax/stratego-front/syn/Stratego-Core.sdf) and implement `TryFrom` for each of them from the generic ATerms. The trait made perfect sense because you can express anything in ATerms and only some are CTrees. 
3. The `preprocessing/ir_v1` code redefines some of the enums and structs to be a bit more specific. Another round of `TryFrom` implementations for each, and you can already see the boilerplate code from the line-count ([1040 LOC](https://gitlab.com/Apanatshka/strs/blob/61784b584f92f68e8f0c4b4451968949556067a3/src/preprocess/ir_v1.rs) to shape the tree a bit more conveniently). 
4. Last section was all about `preprocessing/ir_v2`, which cannot use `TryFrom` because it also passes a context along. Although there are tricks like defining the trait on a tuple (or a little less ad hoc, on a new struct) of the thing and its context, that seems more hacky to me than a custom traversal with plain old methods and functions.

This is.. not ideal. `aterm` is generic, and not the place to add the call-graph. `ctree` and `ir_v1` use `TryFrom` and therefore cannot be easily adapted to pass around a context. `ir_v2` uses a context already, but it's complicated enough as it is and we want to have the call-graph and eliminate dead strategy definitions before we call the `ir_v2` preprocessing step. 

Then again, `ir_v1` has already annoyed me because of the amount of boilerplate around the simple tree transformations. So let's fold that traversal into the one in `ctree`, and change the `TryFrom` implementations into normal methods at the same time. That only takes 4 commits with 2070 insertions and 996 deletions
 ([030c1708](https://gitlab.com/Apanatshka/strs/commit/030c1708956722cdc6b52dbf8360e8d1d665a4ec)-[c562c656](https://gitlab.com/Apanatshka/strs/commit/c562c6567db9a7adf1ef92a002d92908a602db86))...

Now we need a call-graph. So, the definition of strategy definitions becomes:

```rust
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Def<'s> {
    pub name: InternedString<'s>,
    pub sargs: Vec<InternedString<'s>>,
    pub targs: Vec<InternedString<'s>>,
    pub body: Strategy<'s>,
    pub calls: FnvHashSet<InternedString<'s>>,
}
```

The parts we care about during the traversal are: 

1. `CallT`, the strategy call. It gives rise to a strategy reference, which we need.
2. `Let`, which defines strategies, and therefore needs to remove its definitions from the collected strategy references in its body. 
3. `SDefT`, the strategy definition, which takes strategy arguments with names. Those names are also local strategies that are not relevant to the call-graph. 

# The mystery



whatever ¯\\\_(ツ)\_/¯

# Footnotes

[^deforestation]: Deforestation is a compiler optimisation term from functional programming that attempts to remove intermediate data-structures by fusing together operations that normally collect into a data-structure in between. Rust uses the Iterator trait to give you manual control over when you stream your data to the next operator and when you `collect()` into a data-structure again. In functional programming it's called deforestation because you're removing intermediate data-structures through the fusion, and all data-structures are trees. In imperative programming a similar compiler optimisation is loop-fusion. 

