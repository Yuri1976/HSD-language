# HSD — Roadmap

This document describes the order of work for HSD, from where the
project stands today to true self-hosting and beyond. The sequence
is dependency-driven, not time-based. Each phase delivers something
the next phases can rely on.

**The project's North Star: true self-hosting.** A language proves
itself real when it can rebuild itself from its own source — both the
compiler and the runtime written in the language itself. Compiler
self-hosting (Phase 23) arrives first; runtime self-hosting (Phase 35)
closes the circle.

**Self-hosting is incremental.** Not as a final rewrite, but as
milestones interleaved into the language's growth. All four
self-hosting compiler components (lexer, parser, semantic, codegen)
are placed after Phase 15 (variant types), because writing them
without variant types would produce code that needs to be thrown away
and rewritten.

**Beyond self-hosting, into application domains.** Later phases add
the capabilities HSD needs to be applied to real problems: an FFI,
a production backend (LLVM), vector and matrix math, networking, a
web framework, a DataFrame library, graphics bindings, and ML
inference.

**Demonstrators.** Each phase lists small programs (50-200 lines)
that exercise the new features. These validate that the phase is
genuinely complete and form a growing gallery of examples.

**Project Euler.** As HSD matures, Project Euler problems are being
solved and published in `examples/ProjectEuler/`. These serve as
real-world benchmarks and demonstrations of the language in action,
providing concrete speed comparisons against Python, C, and Rust.

---

## Completed phases

~~**Phase 1 - Lexer.**~~ DONE
~~**Phase 2 - Parser.**~~ DONE
~~**Phase 3 - Semantic analyzer.**~~ DONE
~~**Phase 4 - Interpreter.**~~ DONE
~~**Phase 5 - C backend (initial).**~~ DONE
~~**Phase 5b - Actor model in interpreter.**~~ DONE
~~**Phase 5c - Actor model in C backend.**~~ DONE

~~**Phase 6 - ARC.**~~ DONE. Reference counting for all
heap-allocated values. Retain/release inserted by the C backend at
the appropriate places. ARC covers strings (`verba`), lists
(`series[numerus]`), actor pointers, and genus records. Memory
stable at ~0.1 MB across stress tests of 100k+ iterations.
*Demonstrators:* stress test (100k+ strings, memory returns to
baseline); one-million node linked list build and teardown.

**Phase 7 - Error handling** IN PROGRESS - deliberately deferred.
Implementation deferred until after Phase 15. Reason: implementing
`Result`-style error handling before variant types exist would mean
doing the work twice. Go-style multiple return was considered and
rejected for the same reason. Phase 7 will be completed immediately
after Phase 15 lands, using `Result[T, E]` backed by real variant
types.
*Unblocks: writing a compiler that reports errors instead of crashing.*
*Demonstrators:* a calculator REPL that handles invalid input
gracefully; a number-guessing game with proper input validation.

~~**Phase 8 - `genus` end-to-end.**~~ DONE. Record types fully
working in interpreter and C backend. Named-argument construction
(`crea Persona(nome: "Mario", eta: 30)`), field access, field
assignment, structural equality, ARC-tracked heap allocation.
String equality in the C backend uses `strcmp`. Float output format
aligned between interpreter and C backend (`%.7g`).
*Demonstrators:* `examples/rubrica.hsd` (address book);
`examples/geometria.hsd` (Point, Circle, Rectangle with
area/perimeter). Both produce identical output in interpreter and
C backend.

~~**Phase 9 - Module system.**~~ DONE. `affer` fully working:
recursive module loading, path resolution relative to the importing
file, `HSD_PATH` environment variable for global search paths,
circular import detection. Flat namespace - all imported symbols
merge into the global scope (Go-style). Two lexer fixes landed
alongside: CRLF line endings now accepted (Windows compatibility),
tab indentation now accepted (tab = 4 spaces).
*Demonstrators:* `examples/lib/geometria.hsd` (importable geometry
library); `examples/test9c_main.hsd` (multi-file program using it).
*See also:* `HSD-module-system.md` for design rationale and known
limitations.

---

## Active roadmap

**Phase 10 - Standard library bootstrap.**
First real stdlib, written in HSD where possible (with `nativum`
blocks where needed). Focused on what a compiler needs:
- String operations: `longitudo` (length), character access by
  index, `divide` (split), `seca` (trim), `continet` (contains),
  `incipit_a` (starts with), `finit_a` (ends with), concatenation,
  formatting
- List operations: `converte` (map), `filtra` (filter),
  `redige` (reduce), `ordina` (sort)
- Math: `radix` (sqrt), `potentia` (pow), `logarithmus` (log),
  `sinus`, `cosinus`, `fortuna` (random), `absolutum` (abs),
  `minimum`, `maximum`
- Basic I/O: read from stdin line by line

*Unblocks: string-heavy programs, the transformer demonstrator,
the self-hosted lexer.*
*Demonstrators:* a word-count tool (reads stdin, splits into words,
counts and prints); a simple `grep`-like text filter.

**Phase 10b - UTF-8 support.**
Full Unicode support in the runtime and codegen.
`SetConsoleOutputCP(65001)` in the Windows `main` bridge.
UTF-8 validation in the lexer. Correct `verba` length semantics
(characters vs bytes). `lege` reading full Unicode codepoints.
Scheduled here because string handling is already being formalized
in Phase 10.
*See also:* `HSD-known-issues.md`.

**Phase 11 - Polymorphic series + list literals in backend.**
Bring the C backend up to interpreter parity for collection types.
`series[verba]`, `series[realis]`, and `[1, 2, 3]` literals all
compile.
*Demonstrators:* a sorting playground (bubble sort, insertion sort
on `series[verba]`); a Caesar cipher that works on strings.

**Phase 11b - Multidimensional arrays (matrices).**
`series[series[realis]]` and convenience syntax for 2D arrays.
Row/column access, basic matrix operations. Built on Phase 11.
*Unblocks: the transformer demonstrator, linear algebra stdlib.*
*Demonstrators:* matrix multiplication; a simple image represented
as a 2D array of pixel values.

**Phase 12 - File I/O.**
Reading and writing files, working with paths, the basics of the
filesystem.
*Unblocks: the transformer demonstrator (loading model weights),
and the first compiler self-hosting milestone.*
*Demonstrators:* a `cat`-like file printer with line numbering;
a simple log filter that reads a file and prints lines matching
a pattern.

**Phase 12b - MILESTONE: Minimal transformer in HSD.**
A small transformer model written entirely in HSD - no external
libraries, no Python, no PyTorch. Compiled to a standalone C
executable. This is a demonstration milestone, not a production ML
tool. Goals:
- Show HSD viability for numerical/ML-adjacent code
- Understand transformer internals by implementing from scratch
- Demonstrate speed advantage over Python for inference
- Produce a standalone executable with zero dependencies

Inspired by `llama.cpp` - same philosophy, HSD syntax.
Requires: Phase 10 (math stdlib), Phase 11b (matrices),
Phase 12 (file I/O to load weights).
Target use case: lightweight inference on embedded/edge targets
where Python is too heavy and C is too verbose.
*Demonstrators:* a 1-2 layer attention model that completes short
sequences; a benchmark comparing HSD inference speed vs equivalent
Python/NumPy code.

**Phase 13 - Tuples.**
Grammar, semantic analysis, codegen. Lightweight alternative to
`genus` for ad-hoc grouping.
*Demonstrators:* `divmod` (returns quotient and remainder as a
tuple); `min_max` (returns min and max of a list in one pass).

**Phase 14 - Maps.**
Hash maps in the language and the C backend.
*Unblocks: symbol tables in the self-hosted semantic analyzer.*
*Demonstrators:* a word-frequency counter; a simple phone book
with lookup by name.

**Phase 15 - Variant types + pattern matching.**
Treated as a single phase because they belong together. A type can
now have alternatives, and `cum ... est ...` (or similar Latin
syntax - to be decided) destructures them.
*Unblocks: AST as a clean variant type, which makes all four
self-hosting compiler components much nicer. Also unblocks Phase 7
(error handling with Result[T,E]).*
*Demonstrators:* a tiny arithmetic expression evaluator (variant
AST: number, plus, minus, times, divide); a shape area calculator
using variants.

**Phase 7 (completion) - Error handling with Result[T, E].**
Now that variant types exist, `Result[T, E]` is implemented as a
real variant type. Standard library functions become recoverable.
*Demonstrators:* calculator REPL with graceful error handling;
number-guessing game with input validation.

**Phase 16 - First-class functions.**
Function values, closures. Opens the door to higher-order stdlib
functions and cleaner visitor patterns in the self-hosted components.
*Demonstrators:* implement `converte`, `filtra`, `redige` for
`series`; a tiny event-handler dispatcher.

**Phase 17 - Visibility.**
`publicum` / `privatum` on declarations. Real encapsulation.
*Demonstrators:* a small stack library exposing only push/pop/peek
while keeping internal state private.

**Phase 18 - MILESTONE: Lexer in HSD.**
A complete HSD lexer written in HSD: reads a source file, produces
a token stream using variant types, reports errors with file and
line information. Validated by diffing its output against the Rust
lexer on the test corpus.
*Demonstrators:* the lexer itself; a keyword-frequency tool.

**Phase 19 - MILESTONE: Parser in HSD.**
Recursive-descent parser in HSD, building variant-typed AST nodes,
with error recovery.
*Demonstrators:* the parser itself; a pretty-printer for HSD source.

**Phase 20 - MILESTONE: Semantic analyzer in HSD.**
Name resolution, type checking, type inference - all in HSD. Uses
maps for symbol tables, variant types for AST visiting.
*Demonstrators:* the analyzer itself.

**Phase 21 - MILESTONE: Codegen in HSD.**
The C-code generator rewritten in HSD. Reads an AST, walks it,
produces C code. Stresses string building and recursion.
*Demonstrators:* the codegen itself; a pretty-printer that takes
an AST and emits formatted HSD source.

**Phase 22 - Real actor concurrency.**
A scheduler (pool of OS threads, work-stealing), per-actor
mailboxes, asynchronous message delivery. The actor model finally
delivers on its promise.
*Demonstrators:* ping-pong between two actors running concurrently;
a producer/consumer pipeline; a parallel word-count.

**Phase 23 - MILESTONE: Compiler self-hosting complete.**
All four compiler components (lexer, parser, semantic, codegen)
running in HSD. The Rust compiler becomes the bootstrap compiler.
The HSD compiler is in HSD.
*Demonstrators:* the HSD compiler compiles its own source; output
is identical to the Rust-compiled version on the test corpus.

**Phase 24 - LLVM backend.**
A second backend emitting LLVM IR. Selected via `--backend=llvm`;
the C backend stays as `--backend=c` for development.
*Demonstrators:* benchmark C vs LLVM backends; a cross-compilation
demo.

**Phase 25 - Foreign Function Interface (FFI).**
A clean way to declare and call functions from external C libraries.
*Unblocks: graphics, networking, ML inference bindings.*
*Demonstrators:* call `printf` and `sqrt` from libc; bind to
libcurl for a one-line HTTP GET tool.

**Phase 26 - Enhanced `nativum` and low-level control.**
SIMD intrinsics, manual memory layout, tight loops without ARC
overhead.
*Demonstrators:* SIMD-accelerated vector add; a manual memory pool.

**Phase 27 - Math and linear algebra stdlib.**
Vectors (`vec2`, `vec3`, `vec4`), matrices (`mat2`-`mat4`),
quaternions, transformations. Built on Phase 26 SIMD.
*Demonstrators:* ray-sphere intersection; 3D camera transform.

**Phase 28 - Networking stdlib.**
TCP/UDP sockets, DNS, both synchronous and actor-based async.
*Demonstrators:* echo TCP server; chat client/server pair.

**Phase 29 - HTTP/web framework built on actors.**
Small HTTP server where each request is handled by an actor.
*Demonstrators:* hello-world web server; JSON todo API; chat
server with multiple connected clients as actors.

**Phase 30 - DataFrame stdlib.**
Tabular data, typed columns, vectorized operations.
*Demonstrators:* mini-Pandas tool (load CSV, filter, groupby);
parallel ETL pipeline using actors.

**Phase 31 - Graphics bindings.**
SDL2 (windowing, 2D, input) and OpenGL (3D).
*Demonstrators:* bouncing ball; Conway's Game of Life; wireframe
3D cube viewer; Gaussian splat viewer on CPU (loads a `.ply` file
of 3D Gaussians and renders them - a serious test of SIMD, math
stdlib, and graphics).

**Phase 32 - ML inference bindings.**
FFI bindings to ONNX Runtime and GGUF loaders. Using ML, not
training ML.
*Demonstrators:* image classifier (MobileNet via ONNX); sentiment
analyzer; tiny LLM inference via GGUF.

**Phase 33 - Tooling.**
Formatter, `hsd build`/`hsd run` wrapper, improved REPL, language
server for editor integration.

**Phase 34 - User documentation.**
Language reference, tutorial, worked examples.

**Phase 35 - MILESTONE: Runtime self-hosting - true self-hosting
complete.**
The runtime library (`runtime.c` today) rewritten in HSD, using
`nativum` blocks where the bottom must touch the operating system.
ARC, string handling, actor scheduler, file I/O - all in HSD.
This is where HSD depends on nothing it did not write itself,
modulo OS calls.
*Demonstrators:* ARC retain/release in HSD with `nativum` for
atomic primitives; `hsd_lege` in HSD; identical output on the
entire test corpus.

**Phase 36 - Hermetic mode (optional, Linux only).**
Direct syscalls in assembly, removing the libc dependency entirely.
*Demonstrators:* hermetic hello-world (zero external libs);
hermetic `cat` clone.

---

## Notes

**On the ordering of self-hosting milestones.** All four compiler
self-hosting phases (18-21) are placed after Phase 15 (variant
types). Writing a lexer or parser without variant types produces
code that would need to be rewritten after Phase 15. The roadmap
reflects this dependency honestly.

**On Phase 7 (error handling).** Deliberately deferred past Phase
15. `Result[T, E]` with real variant types is the right design;
implementing it before would be wasted work.

**On the transformer milestone (Phase 12b).** A demonstration
milestone showing HSD can handle numerical code cleanly, producing
a standalone executable with no dependencies. Target use case:
lightweight inference on edge/embedded targets. Inspired by the
`llama.cpp` philosophy.

**On the dual backend.** The C backend stays as the development
backend (readable, debuggable output) alongside LLVM (production).
The C backend is deprecated only when language work is no longer
revealing codegen-level bugs.

**On compiler vs runtime self-hosting.** Compiler self-hosting
(Phase 23) means the compiler is in HSD. True self-hosting
(Phase 35) means the runtime is too. Only after Phase 35 does HSD
depend on nothing it did not write itself.

**On the package manager.** Module resolution uses `HSD_PATH` today.
A future dev tool will manage a standard global folder
(`~/.hsd/packages/`), with `hsd.toml` as the per-project config.
Post-Phase 23 work. See `HSD-known-issues.md`.

**On GPU computing.** HSD targets CPU. The `nativum` FFI mechanism
allows calling external CUDA libraries - the same approach used by
Python (PyTorch), Rust (cudarc), and C#. Native GPU kernel
compilation is out of scope.
