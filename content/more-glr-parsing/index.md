+++
title = "More GLR Parsing"
date = "2025-02-08"
draft = true
taxonomies.tags = ["theory of computation", "automata", "context-free grammar", "pda", "parsing"]
+++


## Cubic Time with BRNGLR

From the sections on (RN)GLR parsing, we've learned that we can parse inputs according to any context-free grammar we like if we're willing to use a polynomial time algorithm, that's also dependent on the longest RHS of a grammar rule. We saw the more expensive operation in the middle of the parser would take worst case $O(n^m)$ time where $n$ is the input length and $m$ is the length of the longest RHS. Since that expensive operation was just part of reducing, which is done in a loop over the input of length $n$, the actual worst case performance of RNGLR is $O(n^{m+1})$.

And yet, there are other algorithms for parsing with general context-free grammars that take only _cubic_ (i.e. $O(n^3)$) time. These are [Earley](https://en.wikipedia.org/wiki/Earley_parser) and [CYK](https://en.wikipedia.org/wiki/CYK_algorithm). The downside of CYK is that you first need to transform your grammar to Chomsky normal form, which makes it a bit harder to connect back to a parse tree for the original grammar, and the constants hidden in the big-O notation aren't great, so average performance suffers. Earley is linear for LR(k) grammars, quadratic for unambiguous grammars, and cubic for anything else. But it again has a large constant coefficient which makes it compare poorly with e.g. linear time parsing algorithms. RNGLR is optimised to have fairly low constant coefficients, is linear for LR(k) grammars, but can go to any polynomial for arbitrary grammars. If we take inspiration for CYK and transform our input grammar to a grammar with only rules of RHS max 2, we can get cubic time. This is possible for any context-free grammar, but it'll increase the size of the grammar quite a bit, adding extra non-terminals and rules, which also increases the stack activity of the parser.

You know, it's still kind of interesting to think about what it does to the parsing process to change all the grammar rules to be at most length 2. If during a reduce action of the grammar, we're guaranteed rules of length 0-2: 0 is a nullable rule which we've just looked at, 1 means the reduction can be performed without search because we already have the first edge of the path when we schedule the reduce, and 2 means we only need to look one step further.

1. Work through examples 6.3 and 6.4 to show the effect of binarising the grammar (using automata instead of tables)
2. Explain the on-the-fly reduction path factorisation (section 6.5)?
