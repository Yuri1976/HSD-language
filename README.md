# HSD — Hic Sunt Dracones

> *"Here be dragons"* — the note medieval cartographers placed on the
> unexplored edges of their maps. A new programming language is exactly
> that: uncharted territory.

HSD is a general-purpose programming language with a **Latin-keyword
syntax**. The goal: as simple and flexible as Python, as fast and powerful
as C. Multi-paradigm, statically typed, compiled.

> **Status:** early development. Phase 1 (the lexer) is complete.
> This is a long-term project built step by step.

---

## Design philosophy

Three principles guide every decision:

1. **One thing, one way** — for each problem, one obvious solution.
2. **Zero-cost abstractions** — high-level constructs compile to the same
   code you would write by hand in C.
3. **Static types, but invisible** — you write `sit x = 5`, not the type.
   Type inference does the work; the simplicity is in not *writing* types,
   not in their absence.

## Key characteristics

- **Latin keyword syntax** — `munus` (function), `sit` (variable),
  `si`/`aliter` (if/else), `dum` (while), and so on.
- **Indentation-based blocks** — no curly braces, like Python.
- **Compiled** — HSD transpiles to C, reusing mature C compilers
  (GCC/Clang) for native speed. An LLVM backend may follow.
- **Memory management** — Automatic Reference Counting (ARC) by default,
  with an explicit `nativum` opt-out to manual memory for hot sections.
- **Concurrency** — the actor model: isolated actors that communicate by
  messages, with no shared memory and therefore no data races by default.
- **Multi-paradigm** — imperative core, lightweight OOP via `genus`,
  functional features.

## A taste of HSD

```hsd
# Compute a factorial

munus factorial(n: numerus) -> numerus
    si n <= 1
        redde 1
    aliter
        redde n * factorial(n - 1)

munus principale() -> nihil
    sit result = factorial(5)
    scribe("Factorial of 5: ", result)
```

With actors:

```hsd
nuntius Increment

actor Counter
    sit value: numerus = 0

    accipe Increment
        value = value + 1

munus principale() -> nihil
    sit c = crea Counter
    mitte Increment ad c
```

---

## Building and running

The compiler is written in **Rust**. At this stage only the lexer exists;
it reads a `.hsd` file and prints the tokens it produces.

```sh
rustc lexer.rs
./lexer examples/factorial.hsd
```

A C compiler (GCC or Clang) will become a requirement later, once the
C backend is in place.

---

## Roadmap

The compiler is built in phases. Each phase is decoupled from the next.

| Phase | Goal | Status |
|-------|------|--------|
| 0 | Design on paper — EBNF grammar | done |
| 1 | Lexer — source text to tokens | done |
| 2 | Parser — tokens to an AST | next |
| 3 | Semantic analysis — name resolution, type inference | planned |
| 4 | Tree-walking interpreter — the language runs | planned |
| 5 | Intermediate representation + C backend | planned |
| 6 | Standard library + tooling | planned |

The pipeline:

```
source .hsd
  -> lexer        (Phase 1)
  -> parser       (Phase 2)
  -> semantics    (Phase 3)
  -> lowering/IR  (Phase 5)
  -> C backend    (Phase 5)   ->  emits a .c file
                                  -> GCC/Clang -> native executable
```

## Long-term vision

HSD's distinctive thesis: Latin is an **inflected** language — a word's
form carries its grammatical role. The plan is to make this a real feature
through the **macro system**, letting users define constructs that exploit
cases and verb forms. The language *core* stays small and fixed; the
inflectional richness lives in metaprogramming, opt-in for those who want
it. No English-based language can imitate this.

---

## Project structure

```
.
├── lexer.rs                  # Phase 1 — the lexer
├── examples/                 # sample .hsd programs
│   └── factorial.hsd
├── HSD-grammatica-EBNF.md     # Phase 0 — the formal grammar
├── HSD-scheda-progetto.md     # design sheet: philosophy, roadmap, vision
├── .gitignore
└── README.md
```

---

## License

Not yet chosen. To be decided before the repository is made public.
