# HSD — Project Overview

> **HSD** — short for *Hic Sunt Dracones*, "here be dragons", the Latin
> phrase that medieval cartographers wrote across the unmapped regions
> of their maps. A general-purpose, multi-paradigm language with Latin
> keywords: meant to read like Python, compile like C, with a
> first-class actor model for concurrency.

---

## About this project

HSD is a personal project I started more than five years ago. The first
version was written in C — and, like many language experiments built
in C by a single person, it slowly tangled itself in memory bugs and
half-finished ownership rules until I put it down.

I came back to it recently and rewrote most of the compiler in Rust.
The C parts that survived are the runtime — the small library of
support functions that compiled HSD programs link against. The Rust
parts handle everything else: lexing, parsing, type-checking,
interpretation, and code generation.

The project today is in a **didactic state**: it works end-to-end. A
`.hsd` source file is lexed, parsed, type-checked, and either executed
by a tree-walking interpreter or compiled to a native executable
through a C backend. It is not production-ready, and several pieces of
the original design — most honestly, the automatic memory management —
are still on paper, not in code. But it runs, and the dragons are a
little more mapped than they were five years ago.

HSD draws on concepts from many languages, brought together in a
deliberate way, in pursuit of a language that is fast, simple, and —
hopefully — powerful and solid.

This document is the project sheet: the *why* and the *what* of HSD.
The formal grammar lives in a separate document, as do the language
comparisons and the worked examples.

---

## Design philosophy

Three guiding principles that shape every decision in the language.

**1. One thing, one way.** For each problem, an obvious solution
(borrowed from Python). Avoid multiple equivalent ways of doing the
same thing.

**2. Zero-cost abstractions.** High-level constructs compile to the
same code you would write by hand in C. Convenience does not come at
the cost of performance.

**3. Static types, but invisible.** The user writes `sit x = 5`, not
`numerus x = 5`. The simplicity of Python is in *not writing* the
types — not in their absence. Type inference does the work; the type
system stays watchful in the background.

---

## The language at a glance

### Keywords

| Concept              | HSD                  | Latin root meaning           |
|----------------------|----------------------|------------------------------|
| variable             | `sit`                | "let it be"                  |
| constant (immutable) | `fixum`              | "fixed"                      |
| function             | `munus`              | "task, function"             |
| return               | `redde`              | "give back!"                 |
| if                   | `si`                 | "if"                         |
| else if / else       | `aliter si` / `aliter` | "otherwise (if)"           |
| conditional loop     | `dum`                | "while"                      |
| iterative loop       | `per ... in`         | "for ... in"                 |
| break / continue     | `frange` / `perge`   | "break!" / "carry on!"       |
| true / false         | `verum` / `falsum`   | "true" / "false"             |
| and / or / not       | `et` / `vel` / `non` | "and" / "or" / "not"         |
| null value           | `nihil`              | "nothing"                    |
| record type          | `genus`              | "kind, class"                |
| module import        | `affer`              | "bring!"                     |
| actor                | `actor`              | "actor, doer"                |
| message handler      | `accipe`             | "receive!"                   |
| message type         | `nuntius`            | "messenger"                  |
| send a message       | `mitte ... ad`       | "send ... to"                |
| spawn an actor       | `crea`               | "create!"                    |
| self                 | `ipse`               | "self"                       |
| low-level opt-out    | `nativum`            | "native"                     |
| comment              | `#`                  | —                            |

That is 27 keywords in total — among the leanest cores around (Go has
25, Lua 22, Python 35, C around 44, Rust over 50). The choice is
deliberate: standard-library functions like `scribe` ("write!"),
`lege` ("read!"), and `numera` ("count!") are not keywords. They are
pre-declared names in the semantic analyzer. This lets the standard
library grow without ever touching the lexer or parser.

### Primitive types

| Type     | HSD       | Equivalent to    |
|----------|-----------|------------------|
| integer  | `numerus` | `int64`          |
| float    | `realis`  | `double`         |
| boolean  | `veritas` | `bool`           |
| string   | `verba`   | UTF-8 string     |
| list     | `series`  | dynamic array    |
| nothing  | `nihil`   | `void` / `null`  |

### Example program

```romanes
munus factorial(n: numerus) -> numerus
    si n <= 1
        redde 1
    aliter
        redde n * factorial(n - 1)

munus principale() -> nihil
    fixum limit = 6
    per i in numera(1, limit)
        scribe("factorial(", i, ") = ", factorial(i))
```

A few things to notice in this snippet:

- **Indentation-based blocks**, like Python: no curly braces.
- **Type annotations are required on function signatures**
  (`n: numerus`, `-> numerus`) and **inferred on locals**
  (`fixum limit = 6`).
- `principale` is the program's entry point, by convention.
- `numera`, `scribe` are standard-library functions, not keywords.

---

## Concurrency: a language of actors

HSD's distinctive choice for concurrency is to put actors in the
grammar, not in a library. An `actor` is a unit of state with handlers
that respond to messages; messages are typed (`nuntius`), sent with
`mitte ... ad`, received with `accipe`. **No two actors share memory.**

This is borrowed openly from **Erlang**, the language that has run
telecommunications systems since the 1980s and proved a principle now
widely accepted in concurrent design: *the safest way to avoid memory
races is to make shared memory impossible at the language level*. If
two units cannot touch the same memory, they cannot race on it. The
problem is not solved — it is removed.

Where HSD differs from Erlang is the **opt-out**: the `nativum` block
is an explicit door back into shared memory, for code that needs it
and accepts the responsibility. Erlang allows no escape; HSD prefers
a clear escape with a clear cost. The lineage HSD claims is broad —
Hewitt's original actor model (1973), Smalltalk's "objects that
receive messages", Erlang and Elixir, more recently Pony with its
compile-time guarantees against data races.

---

## Compiler architecture

The phases are decoupled: changing one piece does not touch the
others.

```
.hsd source
    │
    ▼  Lexer              → token stream
    ▼  Parser             → AST
    ▼  Semantic analysis  → typed AST (name resolution + type check)
    │
    ├──→ Tree-walking interpreter   → direct execution
    │
    └──→ C backend (codegen)        → .c file
                                        ↓
                                  gcc / cl → native executable
```

Two execution paths share the same typed AST. The **interpreter** is
slow but fully featured. The **C backend** is fast but covers slightly
less of the language. The interpreter doubles as the **oracle**: when
the two diverge, the interpreter is what the language *should* mean,
and the backend is what needs fixing.

There is no separate intermediate representation. The codegen walks
the AST directly. Adding an IR was on the original plan, but for the
current scope it has not been needed — the C compiler downstream does
enough optimization that adding our own layer would have been work
without payoff. The slot is left open for the future, if and when it
matters.

---

## Implementation status

An honest accounting.

**Fully implemented:**
- Lexer with indentation tracking (`INDENT`/`DEDENT`).
- Recursive-descent parser, with a Pratt-style sub-parser for
  expressions.
- Semantic analyzer: name resolution, full type checking with
  inference.
- Tree-walking interpreter: variables, control flow, functions with
  recursion, actors (synchronous), I/O via `lege`/`scribe`.
- C backend: functions, numbers, booleans, control flow, `verba`
  variables, `series[numerus]` lists, the `per` loop, actors.

**Designed but not yet implemented:**
- **ARC (automatic reference counting).** The memory model promised in
  the original design. In the interpreter, Rust manages memory under
  the hood. In the C backend, every allocation currently leaks until
  the program ends. Small programs run fine; long-running programs
  would not.
- **`genus` (records).** The grammar accepts them and the analyzer
  verifies their fields, but neither the interpreter nor the backend
  executes them. The runtime data structure is absent.
- **True concurrency in actors.** The actor model is implemented
  synchronously in both interpreter and backend: `mitte` runs the
  handler immediately, on the same thread. No mailbox, no scheduler,
  no parallelism.
- **List literals like `[1, 2, 3]`.** The interpreter supports them;
  the C backend does not.
- **`series` of types other than `numerus`.** `series[verba]`,
  `series[realis]` are accepted by the analyzer; the C backend supports
  only `series[numerus]` so far.

**Planned in the design but not yet started:**
- Tuples, maps (dictionaries), sets.
- Error handling (no equivalent of `try`/`catch` or `Result<T, E>` yet).
- A real standard library: string and list operations, math, file I/O,
  time.
- A proper module system (`affer` is currently a placeholder).
- Visibility (public/private) on declarations.

---

## Three decisions to get right

These are timeless: they shape every other choice in the language.

**1. Memory management.** The plan was always **ARC** (automatic
reference counting): simple, predictable, deterministic. No tracing
garbage collector, no manual `free` everywhere. As of now, this is
*designed* but not *implemented* — the most significant gap between
the project on paper and the project in code. The `nativum` block is
the planned escape hatch for code that needs manual control.

**2. Type inference.** This is what makes the language *feel* simple
while staying compiled and strict. Inference is implemented and works
across declarations, function calls, and list elements. Without it,
HSD would feel like C with Latin makeup — the opposite of the goal.

**3. The discipline to say no.** Every feature added for "power"
erodes "simplicity". The difficulty is not technical: it is the design
discipline to leave things out. HSD has 27 keywords because of this
principle. Resisting the urge to add more is a constant tension.

---

## Long-term vision

Two ideas that give HSD its own direction. Both are visions, not
features: they were sketched in the original design and have not
entered the implementation.

### Latin inflection as a metaprogramming layer

Latin is an *inflectional* language: the form of a word carries its
grammatical role. No language built on English keywords can imitate
this — English has no cases. That alone makes Latin a genuinely
original space to design in.

Ideas worth exploring some day (none of them in the core):

- **Cases for the roles of data.** Already used in embryo:
  `mitte Incrementa ad c` mixes accusative ("Incrementa") and a
  prepositional direction ("ad c"). Other roles could follow.
- **Imperative vs indicative.** `scribe` (imperative) executes; an
  indicative form could express a query or a predicate.
- **Singular vs plural.** `numerus` is an integer, `numeri` could be
  a list of integers, directly in the type name.
- **Comparatives and superlatives.** `magnus` / `maior` / `maximus`
  for expressing levels (priorities, optimization grades).
- **Verbal tenses for the temporality of evaluation.** `computa`
  (now), `computabit` (lazy, future), `computavit` (memoized, past).

**The trap.** The more you exploit inflection, the harder the language
becomes to write *correctly*: every keyword must be declined in the
right form. This is in direct tension with "one thing, one way".

**The reconciliation.** Stratify. The **core** of HSD uses fixed
forms, one per concept (as in the EBNF grammar). The richness of
inflection would live in a future **macro / DSL layer**: those who
want declensions can define their own constructs; those who don't,
write plain HSD.

This is a vision for later, not for now.

### Opt-in strictness levels

The second distinctive idea. Most languages pick a side: either
everything is strict (Rust) or everything is permissive (Python).
HSD's ambition is a middle path where **each module declares its own
strictness level**.

The idea: one module asks for stringent checks (strict types, no
mutable globals, strong actor rules), another module is "loose" — fast
to prototype in. The programmer picks the level of strictness per
context, instead of being subjected to a single project-wide setting.

No mainstream language offers per-module declarable strictness in this
form. The closest relatives are compiler pragmas in C and the
optimization declarations of Common Lisp, but they govern lower-level
things — warnings, optimization levels — rather than *which semantic
checks are active*. HSD's notion is more ambitious: strictness is part
of a module's contract, not a tuning knob.

**The constraint.** A strictness level is, concretely, *which checks
of the semantic analyzer are active*. This feature cannot be designed
or implemented until the baseline semantic analysis is complete and
trusted — which it now is. The design can begin next; it has not yet.

**Already decided for the core.** Global variables are allowed only
as constants (`fixum`). No mutable global state — that would be
shared mutable state, in direct conflict with the actor model. This
rule is already enforced today, regardless of any future strictness
layer.

---

## Resources & inspiration

### Books on building languages

- **Robert Nystrom — *Crafting Interpreters*** (free online at
  *craftinginterpreters.com*). Builds a language end-to-end, first as
  an interpreter, then as a bytecode VM. The most practical book on
  the topic.
- **Thorsten Ball — *Writing an Interpreter in Go***. Same spirit,
  different host language, very approachable.
- **Andrew Appel — *Modern Compiler Implementation in ML*** (and its
  Java / C variants). The classic academic reference, for going deeper
  on type systems and code generation.
- **Niklaus Wirth — *Compiler Construction*** (free PDF). A short
  classic by the creator of Pascal, for the elegance of a small
  compiler done right.

### Languages that shaped HSD

- **Erlang** and **Elixir** — for the actor model and the
  isolation-first approach to concurrency. Proof that "no shared
  memory" can power real systems for decades, from telecom switches
  to modern messaging platforms (WhatsApp, Discord).
- **Smalltalk** — for the original insight that programs are best
  thought of as objects sending and receiving messages, and for the
  discipline of a tiny core (just six keywords).
- **Python** — for indentation-based syntax and the principle that
  code is read more often than it is written.
- **Rust** — for the discipline of static types with inference, and
  for showing that systems programming and ergonomics are not enemies.
- **Swift** — for ARC as a production-grade technology and not a
  research idea. Proof that automatic reference counting scales to
  real systems.
- **Nim** — for the transpile-to-C architecture: small core, large
  standard library, runtime small enough to read. The closest cousin
  to HSD in execution model.
- **Lua** — for what a 22-keyword core can do, and for being easy to
  embed in larger systems.
- **Pony** — for showing how far the actor model can be pushed with
  static guarantees against data races. Aspirational, not imitated.
- **Go** — for the lesson that a small, opinionated language can be
  more productive than a kitchen-sink one. Different goals from HSD,
  but the same restraint.

### Foundational papers and ideas

- **Carl Hewitt et al. (1973) — *A Universal Modular Actor Formalism
  for Artificial Intelligence***. The original paper that introduced
  the actor model. Worth reading at least once for the historical
  shape of the idea.
- **Tony Hoare (1978) — *Communicating Sequential Processes***. The
  other foundational concurrency model. Different from actors in the
  details, sharing the same principle: communicate, do not share.
- **Robin Milner — work on the π-calculus**. The formal underpinning
  of much of modern concurrency theory. Heavy, but the ideas trickle
  down into language design.
