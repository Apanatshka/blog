

:- | :-
{%latex%} S = A\ a\ b\ A\ b\ a {%endlatex%} | {%latex%} \text{(1)} {%endlatex%}
{%latex%} A = a {%endlatex%}                | {%latex%} \text{(2)} {%endlatex%}
{%latex%} A = A b {%endlatex%}              | {%latex%} \text{(3)} {%endlatex%}

Remember the PDA of these rules looked like this:

{% digraph Single PDA using the automata from the grammar rules (repeated from before) %}
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
A₃₀ [style=filled, fillcolor="#aaa"];
A₃₀ -> S₁₀ [minlen=2];
S₁₀ -> A₂₀ [minlen=2];
}
subgraph {
rank=same;
edge [style=invis];
A₂ [style=filled, fillcolor="#ddd"];
A₃₁ [style=filled, fillcolor="#aaa"];
A₃₁ -> S₁₁ [minlen=2];
S₁₁ -> A₂ [minlen=2];
}
subgraph {
rank=same;
edge [style=invis];
A₃ [style=filled, fillcolor="#aaa"];
A₃ -> S₁₂ [minlen=2];
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

subgraph {
A₃₀ -> A₃₁ [label="a"];
A₃₁ -> A₃ [label="b"];
}

S₁₀ -> A₂₀ [label="↓S₁₁  ", weight=0.5];
A₂ -> S₁₁ [label="\n\n       ↑S₁₁", weight=0.5];
S₁₀ -> A₃₀ [label="↓S₁₁  ", weight=0.5];
A₃ -> S₁₁ [taillabel="↑S₁₁", labelangle=50, labeldistance=1.5, weight=0.5];

S₁₃:sw -> A₂₀:ne [taillabel="↓S₁₄", labelangle=-35, labeldistance=3, weight=0.5];
A₂ -> S₁₄ [taillabel="↑S₁₄", labelangle=40, labeldistance=2, weight=0.5];
S₁₃:nw -> A₃₀:se [taillabel="↓S₁₄", labelangle=-50, labeldistance=2, weight=0.5];
A₃ -> S₁₄ [taillabel="↑ S₁₄", labelangle=40, labeldistance=2, weight=0.5];
{% enddigraph %}

When we looked at this style of PDA before, we took it as a partial view of an LL parser, where the non-determinism was resolved by an LL table we could derive from this. Resolving non-determinism in a PDA is important to keep down the cost of running it, because unlike an NFA, you can't just remove non-determinism in a PDA by expanding the number of states to model being in multiple states. That's because the PDA also has the stack as extra state, and there are infinite configurations of that stack...

### Single stack non-deterministic PDAs

Let's look at the above PDA as a non-deterministic PDA with one important restriction: the stack must be the same for each state we're in simultaneously. This kind of non-deterministic PDA is not super expensive to compute on, it's basically as expensive as an NFA, with one stack kept on the side.

What's the effect of this kind of view on the PDA? Well, we get to go into any number of rules simultaneously, and down to the leftmost child, which is what we need for LR parsing. We also get to try multiple rules until one of them continues and another one ends, because that's where the stacks would diverge. So in LR we make the decision of which rule to choose much later than in LL, namely we read as much of the input as possible before deciding which rule applies, restricted by the 'same stack' limitation. In the example we can use something akin to NFA-to-DFA conversion now:

{% digraph Same PDA but with merged nodes %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;

start1 [shape=none, label="", width=0];
S₁ [shape=doublecircle, width=0.4];

subgraph {
rank=same;
rankdir=LR;
edge [style=invis];
A0 [shape=box, label=<A₂₀<br/>A₃₀>, style="rounded,filled", fillcolor="#aaa"];
A0 -> S₁₀ [minlen=2];
}
subgraph {
rank=same;
edge [style=invis];
A1 [shape=box, label=<A₂<br/>A₃₁>, style="rounded,filled", fillcolor="#aaa"];
A1 -> S₁₁ [minlen=2];
}
subgraph {
rank=same;
edge [style=invis];
A₃ [style=filled, fillcolor="#aaa"];
A₃ -> S₁₂ [minlen=2];
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
A0 -> A1 [label="a"];
A1 -> A₃ [label="b"];
}

S₁₀ -> A0 [label="↓S₁₁  ", weight=0.5];
A1 -> S₁₁ [label="↑S₁₁  ", weight=0.5];
A₃ -> S₁₁ [taillabel="↑S₁₁", labelangle=60, labeldistance=2, weight=0.5];

S₁₃ -> A0 [taillabel="↓S₁₄", labelangle=-35, labeldistance=3, weight=0.5];
A1 -> S₁₄ [taillabel="↑S₁₄", labelangle=40, labeldistance=2, weight=0.5];
A₃ -> S₁₄ [taillabel="↑ S₁₄", labelangle=40, labeldistance=2, weight=0.5];
{% enddigraph %}

Hmmmm, in state A2/A31 we still need a lookahead of 2 to choose between A3 and S14. But if our stack has an S11 on it, we only need a lookahead of 1 to choose between A3 and S11. This shows why we may want to distinguish between the first and second occurrence of A in rule (1), just like we did in the LL case. But in the LR case we'd then want to end up with separate states in the automaton to be able to distinguish this, but is that really worth increased automaton size?

### (Left) Recursion

Let's keep that separate rules for separate occurrences in mind though, because it will be useful in the next step. Because left recursion still breaks these automata: pushing onto the stack whenever you go into one or more rules means you're still predicting how far down into the parse tree you are, which isn't true bottom-up parsing, and isn't really effective:

:- | :-
{%latex%} S = S + A {%endlatex%} | {%latex%} \text{(1)} {%endlatex%}
{%latex%} S = a {%endlatex%}     | {%latex%} \text{(2)} {%endlatex%}
{%latex%} A = a {%endlatex%}     | {%latex%} \text{(3)} {%endlatex%}
{%latex%} A = ( A ) {%endlatex%} | {%latex%} \text{(4)} {%endlatex%}

The kind of automaton we were building wouldn't really work because the stacks would be all different for different inputs:

{% digraph PDA for left-recursive grammar illustrating problematic stack usage %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;

start1 [shape=none, label="", width=0];
S₁ [shape=doublecircle, width=0.4];

subgraph {
rank=same;
edge [style=invis];
S₁₀ -> S₂₀;
}

subgraph {
rank=same;
edge [style=invis];
S₁₁ -> S₂;
}

subgraph {
rank=same;
edge [style=invis];
A₃₀ -> S₁₂;
S₁₂ -> A₄₀;
}

subgraph {
rank=same;
edge [style=invis];
A₃ -> S₁;
S₁ -> A₄₁;
}


subgraph {
start1 -> S₁₀;
S₁₀ -> S₁₁ [label="S", fontcolor="#01010140", color="#01010140"];
S₁₁ -> S₁₂ [label="+"];
S₁₂ -> S₁ [label="A", fontcolor="#01010140", color="#01010140"];
}

subgraph {
start1 -> S₂₀;
S₂₀ -> S₂ [label="a"];
}

subgraph {
A₃₀ -> A₃ [label="a"];
}

subgraph {
A₄₀ -> A₄₁ [label="("];
A₄₁ -> A₄₂ [label="A", fontcolor="#01010140", color="#01010140"];
A₄₂ -> A₄ [label=")"];
}

S₁₀ -> S₁₀ [label="↓S₁₁", weight=0.5];
S₁₀ -> S₂₀ [label="↓S₁₁", weight=0.5];
S₁ -> S₁₁ [headlabel="↑S₁₁", labeldistance=2, labelangle=40, weight=0.5];
S₂ -> S₁₁ [label="↑S₁₁", weight=0.5];

S₁₂ -> A₃₀ [label="↓S₁", weight=0.5];
S₁₂ -> A₄₀ [label="↓S₁  ", weight=0.5];
A₄:n -> S₁:e [label="↑S₁", weight=0.5];
A₃ -> S₁ [label="↑S₁", weight=0.5];

A₄₁ -> A₃₀ [headlabel="↓A₄₂", labelangle=40, labeldistance=2.5, weight=0.5];
A₄₁ -> A₄₀ [xlabel="↓A₄₂", weight=0.5];
A₄:nw -> A₄₂:new [xlabel="↑A₄₂", weight=0.5];
A₃ -> A₄₂ [headlabel="↑A₄₂", labelangle=-30, labeldistance=2.5, weight=0.5];
{% enddigraph %}

We need a better way to use the stack if we want to do bottomup parsing, one where we don't push on the stack when we do recursive calls.

One of the big reasons to even push something onto the stack, is to know where to go back to when we are done with a rule. If we take our idea from before and create a new instance of each rule of a sort for each occurrence of that sort in the grammar, we will statically know where we came from. So that's good, as long as we ignore automaton size for a second. We can't entirely get rid of using the stack though, in our example grammar sort A needs balanced brackets after all:

{% digraph PDA for left-recursive grammar illustrating problematic stack usage %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;

start1 [shape=none, label="", width=0];
S1 [label=S₁, shape=doublecircle, width=0.4];

subgraph {
rank=same;
edge [style=invis];
S10 [label=S₁₀];
S20 [label=S₂₀];
S10 -> S20;
}

subgraph {
rank=same;
edge [style=invis];
S11 [label=S₁₁];
S2 [label=S₂];
S11 -> S2;
}

subgraph {
rank=same;
edge [style=invis];
A30 [label=A₃₀];
S12 [label=S₁₂];
A40 [label=A₄₀];
A30 -> S12;
S12 -> A40;
}

subgraph {
rank=same;
edge [style=invis];
A3 [label=A₃];
S1 [label=S₁];
A41 [label=A₄₁];
A50 [label=A₅₀];
A60 [label=A₆₀];
A3 -> S1;
S1 -> A41;
A41 -> A50;
A50 -> A60;
}

subgraph {
rank=same;
edge [style=invis];
A42 [label=A₄₂];
A5 [label=A₅];
A61 [label=A₆₁];
A42 -> A5;
A5 -> A61;
}

subgraph {
rank=same;
edge [style=invis];
A4 [label=A₄];
A62 [label=A₆₂];
I1 [style=invis, width=0, height=0.5];
A4 -> I1;
I1 -> A62;
}

subgraph {
start1 -> S10;
S10 -> S11 [label="S", fontcolor="#01010140", color="#01010140"];
S11 -> S12 [label="+"];
S12 -> S1 [label="A", fontcolor="#01010140", color="#01010140"];
}

subgraph {
start1 -> S20;
S20 -> S2 [label="a"];
}

subgraph {
A30 -> A3 [label="a"];
}

subgraph {
A50 -> A5 [label="a"];
}

subgraph {
A40 -> A41 [label="("];
A41 -> A42 [label="A", fontcolor="#01010140", color="#01010140"];
A42 -> A4 [label=")"];
}

subgraph {
A6 [label=A₆];
A60 -> A61 [label="("];
A61 -> A62 [label="A", fontcolor="#01010140", color="#01010140"];
A62 -> A6 [label=")"];
}

S10 -> S20 [label="ε", weight=0.5];
S1 -> S11 [headlabel="ε", labeldistance=2, labelangle=40, weight=0.5];
S2 -> S11 [label="ε", weight=0.5];

S12 -> A30 [label="ε", weight=0.5];
S12 -> A40 [label="ε  ", weight=0.5];
A4:n -> S1:e [label="ε", weight=0.5];
A3 -> S1 [label="ε", weight=0.5];

A41 -> A50 [label="↓A₄₂  ", weight=0.5];
A41 -> A60 [headlabel="↓A₄₂", labelangle=40, labeldistance=2.5, weight=0.5];

A6 -> A42 [headlabel="↑A₄₂", labelangle=50, labeldistance=2, weight=0.5];
A5 -> A42 [label="↑A₄₂  ", weight=0.5];

A61 -> A50 [headlabel="↓A₆₂", labelangle=40, labeldistance=2.5, weight=0.5];
A61:nw -> A60 [xlabel="↓A₆₂", weight=0.5];
A6:nw -> A62:ne [xlabel="↑A₆₂", weight=0.5];
A5 -> A62 [headlabel="↑A₆₂", labelangle=20, labeldistance=2.5, weight=0.5];
{% enddigraph %}



## An intuition for table construction by automaton

:- | :-
{%latex%} S = S\ a\ b\ A\ b\ a {%endlatex%} | {%latex%} \text{(1)} {%endlatex%}
{%latex%} S = a {%endlatex%}                | {%latex%} \text{(2)} {%endlatex%}
{%latex%} A = a {%endlatex%}                | {%latex%} \text{(3)} {%endlatex%}
{%latex%} A = a b {%endlatex%}              | {%latex%} \text{(4)} {%endlatex%}

This is not quite the LL(2) grammar from before. Notice how rule (1) is now left-recursive. This makes a grammar immediately not LL, because the left-recursion would cause an infinite loop. Rule (4) is now non-empty. Now to get to parsing in an LR fashion, we need to figure out what the leftmost leaf node of the parse tree _could_ be. We can expand rule (1) an arbitrary number of times, eventually we'd need to choose rule (2). This makes more sense perhaps when you look at the PDA as we had constructed before for the LL grammar:

{% digraph PDA version 1 for the LR grammar %}
bgcolor="transparent";
node [shape=circle, fixedsize=shape, width=0.5, fontcolor="#010101", color="#010101"];
edge [fontcolor="#010101", color="#010101"];
rankdir=LR;

start1 [shape=none, label="", width=0];
S₁ [shape=doublecircle, width=0.4];
subgraph {
rank=same;
edge [style=invis];
S₂₀ [style=filled, fillcolor="#eee"];
S₁₀ -> S₂₀;
}
subgraph {
rank=same;
edge [style=invis];
S₂ [style=filled, fillcolor="#eee"];
S₁₁ -> S₂;
}
subgraph {
rank=same;
edge [style=invis];
A₃₀ [style=filled, fillcolor="#ddd"];
A₄₀ [style=filled, fillcolor="#ccc"];
F1 [shape=point, color="#010101", margin=0, label="", fontsize=0, width=0, penwidth=1];
A₄₀ -> S₁₃;
S₁₃ -> A₃₀;
A₃₀ -> F1;
}
subgraph {
rank=same;
edge [style=invis];
A₃ [style=filled, fillcolor="#ddd"];
A₄₁ [style=filled, fillcolor="#ccc"];
F2 [shape=point, color="#010101", margin=0, label="", fontsize=0, width=0, penwidth=1];
A₄₁ -> S₁₄;
S₁₄ -> A₃;
A₃ -> F2;
}
subgraph {
rank=same;
edge [style=invis];
A₄ [style=filled, fillcolor="#ccc"];
A₄ -> S₁₅;
}

subgraph {
start1 -> S₁₀;
S₁₀ -> S₁₁ [label="S", fontcolor="#01010140", color="#01010140"];
S₁₁ -> S₁₂ [label="a"];
S₁₂ -> S₁₃ [label="b"];
S₁₃ -> S₁₄ [label="A", fontcolor="#01010140", color="#01010140"];
S₁₄ -> S₁₅ [label="b"];
S₁₅ -> S₁ [label="a"];
}

subgraph {
A₃₀ -> A₃ [label="a"];
}

subgraph {
S₂₀ -> S₂ [label="a"];
}

subgraph {
A₄₀ -> A₄₁ [label="a"];
A₄₁ -> A₄ [label="b"];
}

S₁₀ -> S₂₀ [label="↓S₁₁   ", constraint=false];
S₂ -> S₁₁ [label="↑S₁₁  ", constraint=false];
S₁₀ -> S₁₀ [label="↓S₁₁  ", constraint=false];
S₁:s -> F2:e [arrowhead="none", weight=0.5];
F2 -> F1 [arrowhead="none"];
F1:w -> S₁₁ [label="↑S₁₁", constraint=false];

S₁₃ -> A₃₀ [label="↓S₁₄  ", constraint=false];
A₃ -> S₁₄ [label="↑S₁₄  ", constraint=false];
S₁₃ -> A₄₀ [label="↓S₁₄  ", constraint=false];
A₄ -> S₁₄ [taillabel="↑S₁₄", labelangle=40, labeldistance=2, constraint=false];
{% enddigraph %}

Rather than directly explain how LR works, I'd like to see if we can reconstruct the algorithm from an automaton view. If we start out with the LL automaton we constructed earlier to explain LL table construction, and work our way up from there, perhaps we'll end up at the same point.
