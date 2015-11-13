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
