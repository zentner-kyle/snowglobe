snowglobe
=========

The snowglobe project is an attempt to create agents capable of general
intelligence, even if they're not capable of solving common problems like
computer vision / audio processing / speech synthesis. The general hope here is
that those specfic fields will progress separately, and this system can be
adapted to use them.

What exactly we would like the agents to be capable of is still somewhat
unclear, but here's an initial breakdown:
  - Planning.
    - Scheduling.
    - Cost minimization.
    - Risk reasoning.
  - Learning / Modelling.
    - Causative extraction.
    - Strategic generalization.
    - Emotional prediction.
    - "Self analysis."

The approach of the snowglobe project to achieve this is to attempt to take
genetic algorithms to their natural conclusion. In other words, to create
conditions where multiple agents compete for resources such that the desired
capabilities are developed.

Unfortunately, biologists / anthropologists have relatively little knowledge of
why these capabilities came about naturally.


Learning State Engine (L.S.E.)
==============================

## Summary
The basic idea here is to attempt to combine filters, state machines, and
classifiers into a system that can learn / evolve to solve complex tasks in
novel ways. Ultimately, the system should be designed so that different
(presently undiscovered) classifiers can be integrated into the system,
allowing it to become much more effective.

There are a few (vague) reasons for choosing to partition the general problem this way:
  - Filters are highly efficient and simple, yet capable of achieving difficult
    tasks through feedback.
  - A reasonable number of problems in robotics can be probably be solved
    primarily by composing state machines and filters already. Understanding
    the level of difficulty involved in this design is the goal of the Kinetic
    project.
  - State machines are also highly efficient to evaluate, and there are known
    algorithms for performing various complex operations on them.
  - "Acting like a state machine" is also a trait which indicates an emergent
    system is of interest. Why not bake this assumption into the system?
  - If one assumes that emulating human reasoning in a learning system is a
    feasible way to solve the general problem, then a realistic method of
    doing so is to emulate observable human reasoning processes. States appear
    to be featured in many human reasoning processes, making them a reasonable
    target for emulation.
  - State machines are inherently easier for humans to reason about than most
    computational tools, and are thus potentially central to a method for
    feasibly solving the general problem.
  - Programming and inspecting state machines manually can be reasonably be
    done by humans. The behavior of such a learning system can thus be manually
    assisted and understood, where other system might not.
  - Relatively small additions to finite state machines allow Turing complete
    behavior, which one should expect is required to solve the general problem.

## Design

There are a few approaches we could take here:
  - Evaluate / learn over full Kinetic programs.
  - Evaluate / learn over much more restricted programs.

Furthermore, we could:
  - Hardcode the learning algorithm.
  - Learn the learning algorithm itself.
  - Write a reasonable learning algorithm into the starting system, and allow
    the learning algorithm to run on itself (there is no problem of infinite
    regression here).

For this project, it looks most reasonable to begin with evaluating the most
restricted programs where a reasonable learning algorithm can be encoded.

Kinetic
=======

## Motivation
It has become clear from various experiments (mental and otherwise), that it
would be useful to have a language for describing feedback systems capable of
arbitrarily complex behavior possible in strictly bounded time.

This project began based on the realization that early and non-programmers were
suprisingly capable at programming state machines. However, existing systems do
not provide convenient syntax for describing / reasoning about state machines.
Early programmers are highly reliant on visual indicators of the abstract
concepts they are reasoning about, so no existing programming system can
exploit this approach to teaching programs.

For example, the states in a recursive descent parser are best understood as
the points of execution *between statements*. Transitions are represented as a
combination of if statements, functions, and while loops (unless tail recursion
is provided). Similarly, the states in regular expressions are primarily the
locations *between characters*. Transitions are represented by most characters.

I've encountered few examples of systems where the state is considered
important enough for language integration:
  - Falcon, a little known, multi-paradigm language has support for Stateful
    classes. This is the best treatment of stateful programming I have seen.
  - GSL, a EDSL in Groovy, the subject of a (possibly only) paper on State
    Oriented Programming. It does have an example of convergent evolution with
    Kinetic; it defines `extern` states.
  - Clojure has agents, which act somewhat like state machines. However,
    they're intended only as a concurrency primitive.
  - Scala and Erlang have the actor model, which captures the idea of messages,
    but states themselves must be managed in the usual imperative way.

However, none of these achieve our goals. In particular, none of them have
State-Oriented Programming restrictions. Restrictions are an important aspect
of language design, since they describe what types of advanced generalizations
are guaranteed to be safe in the language.

Functional programming, for example, guarantees that composition relations are
described locally, parallelization is always safe (up to termination),
memoization is always safe, and data-structure sharing is always safe.
Generally, they also guarantee that value and reference semantics are
equivalent, eliminating a major source of confusion.

Similar in spirit restrictions for State-Oriented Programming would allow far
more advanced operations. In particular, consider the restriction of only
having three functions per state: enter, update, and exit. Furthermore,
restrict the use of freshly allocated data in the function where it was
allocated. Preventing recursion (except for some form of inclusion), and
preventing all loops except for loops and simple conditionals. With these
restrictions, it is possible to give a strict upper bound on the runtime of any
program. In fact, the assymptotic runtime of each function can be found
automatically. Guaranteeing determinism, even in combination with
parallelization is also readily achievable.

In fact, from a theoretical point of view this is the most restricted class of
program. Therefore, if a particular class of analysis is possible for any
language whatsoever, it should be possible for this language.

Since we're interested in synthesis of programs anyways, we may as well
consider this class of programs. However, we can still have some tools to make
programming in this environment productive.

## Summary

The type tree of kinetic is as follows:

Here's a BNF grammar describing the type family:

```
  type ::= number | tag type | homogenous | heterogeneous
  homogenous ::= '['type';' count ']'
  heterogeneous ::= '[' type more_types ']'
  more_types ::= type more_types?
  count ::= [0-9]*
  integer ::= (-|+|)[0-9]*
  number ::= precise | imprecise
  precise ::= integer | range | exp | frac
  imprecise ::= (-|+|)[0-9]*'.'[0-9]*'e'(-|+|)[0-9]* // Maybe slightly different, IEEE754 compatible.
  range ::= '[' integer '..' integer ']' | 'saturating' | 'wrapping'
  exp ::= (integer | range | frac) '^' range
  frac ::= (integer | range | exp) '/' exp 
  positive ::= [1-9] positive?
  tag ::= symbol ('|' tag)?
  symbol ::= [A-z_][A-z_0-9]*
```

As you can see, we have a decent numeric tower, based on the idea that we can
do relatively precise type inference thanks to the runtime restrictions. The
above does not specify the inference rules, which are relatively
straightforward to derive, but quite long. These are intended to provide
semantics that are easy for humans and computers to reason about, even if the
actual calculation of the values may not be as fast as possible.

Note that considerable flexibility is snuck into this system by the
introduction of the tag type, which is a polymorphic tag.

However, values are not the primary concern for reasoning in Kinetic, agents
are.

Agents are essentially threads which record a 'state', which is not first
class. Agents themselves are also not first class.

Programs in Kinetic are a directed graph of agents, with additional edges to
'external agents', which are the method of performing IO.

Each state has some local variables specific to that state. A state also has
some initial input variables, a function for performing initialization, and a
function for receiving a message.

Within these functions, new values for local variables can be computed, new
states can be transitioned to, and messages to known agents can be sent.
Because agents are not first class, analysis of "fate sharing" can be
performed, to typecheck the message sends.

When a message send is received, it contains a payload of a known type.

Groups of states can form arbitrary graphs, including loops. It should be
possible to bound the memory usage of queued messages by analyzing the program.

## Design
I would like the system to be usable from a large range of skillsets. In
particular, I'm imagining four forms for any program:
  - Graphical, probably shown in a browser. Visually represents the
    calculations of the functions, as well as the two graphs (the state graph
    and agent graph).
  - Imperative, with for loops expressed using 'for x in y { }' style syntax.
  - Functional, with map and reduce functions style manipulation of values.
  - Array, where operations are described tersely, as in J, K, etc.

Of course, these different disciplines are only cosmetically different for this
restricted class of programs.

Haku
====

## Summary
A system which learns how to plays simple games by example. This is mostly an
experiment focusing on understanding methods of "strategic generalization."
The main goal of this project is to match or beat human learning capability on
board games. In other words, the system should need the same number or fewer
example games shown to it compared to a human learning the same game.
In order to achieve this, we're willing to accept some degree of modelling and
priors. However, we would like to understand why those priors exist.

Hopefully, this project will also elucidate some part of how to solve the
causal reasoning problem.

## Design
There are many ways to approach this problem. The most obvious is to attempt
code synthesis over a DSL for solving games. However, quickly becomes very hard
to control and reason about. Furthermore, novel methods are unlikely to arise
from solving the problem in this way.

However, fundamentally we would like to deduce program like outputs.

One discussed approach is hybrid search across a clustering space and policy
space. The main reason this should be tractable is that policy search
terminating with an inconsistent or high-cost (unlikely) policy can be used to
guide search on the clustering space. Furthermore, various amounts of tuning
can be done on the clustering space in order to achieve better learning
performance.

A final approach, which largely diregards conventional methods and feels
attractive to me is to not explicitly search the policy space, and instead
search across set spaces. Essentially, this involves finding consistent
transform sequences for all inputs such that they produce supersets of the
output set consistent with the observed sets. The lack of higher-order sets as
well as the lack of true programmatic description can then be rectified by
performing causal analysis. We'll call this approach set-space-causal-analysis.
This will be the first method attempted, and will be described in more detail
below.

## Implementation of Set Space Causal Analysis
Suppose we begin by featurizing all game boards in a particularly simple way:
We represent a game board as a fixed size group of sets of board indices the
collection of which we call S.
As a simple example, consider this TTT board:

```
O X _
O X _
X _ _
```

Would be mapped to the following sets:

`S[0] = Owned:`
```
# _ _
# _ _
_ _ _
```

`S[1] = Opponent:`
```
_ # _
_ # _
# _ _
```

`S[2] = Blank:`
```
_ _ #
_ _ #
_ # #
```

Conceptually, these are set of X, Y pairs, since this game takes place on a
grid. However, it is also possible to featurize games on hexagonal, 3D, or
radial boards in the same way. Furthermore, all practically sized games can
have their boards represented as small bitmaps.

Now, suppose we have a collection of `i` observed allowed input and output sets
groups.
That is to say, we have `E = [(S, S')]`.

Then, our approach here is relatively simple (compared to explicit search
across a program or policy space): we perform uniform cost search from S to Z,
where for all `S'[i]` there exists a subset `Z[j]` and a superset `Z[k]` (where
`j` and `k` are arbirary).

Critically, the we must find consistent paths (and `j, k`) for all `(S, S')`
pairs in `E`.

The paths along which we're searching are relatively simple. In order to avoid
complicated model choices, we simply maintain a stack `S[i][g]` where `g` is
the `generation` of a set. At each depth of the search, we apply operators to
all `S[i]` such that the sum of the cost of the operators is our search depth.

For any particular search depths, this should leave us with a total ordering of
valid paths for how the sets can change. However, this level of analysis in
insufficient for most games.

For example, consider a simple sliding game like Dao. After performing the
above analysis steps, we can produce the following sets (where `>=S'[i]` is a
member of `Z` which is a super set of `S'[i]` and `<=S'[i]` is a member of `Z`
which is a sub set of `S'[i]`).

```Board: 
X__O
_XO_
_OX_
O__X
```

```S[0]: 
#___
_#__
__#_
___#
```

```S[1]: 
___#
__#_
_#__
#___
```

```S[2]: 
_##_
#__#
#__#
_##_
```

```>=S'[0]
###_
##_#
#_##
_###
```

```<=S'[0]
____
____
____
____
```

```>=S'[1]: 
___#
__#_
_#__
#___
```

```<=S'[1]: 
___#
__#_
_#__
#___
```

```>=S'[2]: 
###_
##_#
#_##
_###
```

```<=S'[2]: 
____
____
____
____
```

However, the only valid moves are those where a piece moves in a straight line
until it hits a boundary (either another piece or the edge of the board).
In other words, the only paths that will only yield valid moves are equivalent
to:

```
S[0] , *L , :
S[0] , *A(1) , S[0][0] *A(2) , S[0][1] *A(2), S[0][2] *A(2), & S[0][3] -> 
|-> S[0][2], S[0][2] *A(1) &-> S[0][1], 
```

This has quickly become a mess, since I don't have a good language for
specifying feedback systems.


