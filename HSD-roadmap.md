# HSD — Roadmap

This document describes the order of work for HSD, from where the
project stands today to true self-hosting and beyond. The sequence
is dependency-driven, not time-based. Each phase delivers something
the next phases can rely on.

**The project North Star: true self-hosting.** A language proves
itself real when it can rebuild itself from its own source — both
the compiler and the runtime written in the language itself.
Compiler self-hosting (Phase 23) arrives first; runtime self-hosting
(Phase 35) closes the circle.

**Self-hosting is incremental.** The order has been deliberately
revised: all four self-hosted compiler components (lexer, parser,
semantic, codegen) are placed after variant types (Phase 15),
because writing them without variant types produces code that would
need to be thrown away. The roadmap reflects this dependency honestly.

**All stdlib names are real Latin.** This is a core design principle.
Every keyword and built-in function uses Latin, not Italian or
English. The existing keywords (sit, fixum, munus, genus, redde,
scribe, lege, dum, per, si, aliter, affer) set the standard. Built-in
function names are abbreviated where no confusion arises, and kept
in full where an abbreviation would be cryptic or clash with a
keyword.

**Project Euler.** Solutions in examples/ProjectEuler/ serve as
living benchmarks and demonstrations. Each problem shows HSD working
on real computation, with compiled C executables providing concrete
speed comparisons against Python and other languages. Problem 1 is
solved and verified in both interpreter and C backend (answer:
233168).

---

## Completed phases

~~Phase 1 - Lexer~~ ✅
~~Phase 2 - Parser~~ ✅
~~Phase 3 - Semantic analyzer~~ ✅
~~Phase 4 - Interpreter~~ ✅
~~Phase 5 - C backend~~ ✅
~~Phase 5b - Actor model in interpreter~~ ✅
~~Phase 5c - Actor model in C backend~~ ✅

~~**Phase 6 - ARC.**~~ ✅ Reference counting for all heap-allocated
values. Covers strings (verba), lists (series[numerus]), actor
pointers, and genus records. Memory stable at ~0.1 MB across stress
tests of 100k+ iterations.

**Phase 7 - Error handling** *(deliberately deferred).*
Deferred until after Phase 15. Go-style multiple return was
considered and rejected — implementing Result before variant types
exist means doing the work twice. Will be completed immediately
after Phase 15 using Result[T, E] backed by real variant types.
*Unblocks: compiler that reports errors instead of crashing.*
*Demonstrators:* calculator REPL with graceful error handling;
number-guessing game with input validation.

~~**Phase 8 - genus end-to-end.**~~ ✅ Record types fully working
in interpreter and C backend. Named-argument construction, field
access, field assignment, structural equality, ARC-tracked heap
allocation. String equality via strcmp. Float output aligned
between interpreter and C backend (%.7g).
*Demonstrators:* examples/rubrica.hsd; examples/geometria.hsd.

~~**Phase 9 - Module system.**~~ ✅ affer fully working: recursive
module loading, HSD_PATH support, circular import detection. Flat
namespace (Go-style). Two lexer fixes: CRLF line endings and tab
indentation (tab = 4 spaces).
*Demonstrators:* examples/lib/geometria.hsd; examples/test9c_main.hsd.
*See also:* HSD-module-system.md, HSD-known-issues.md.

~~**Phase 10 - Standard library bootstrap.**~~ ✅ *Completed.*
First real stdlib, implemented as built-in functions. Names are
real Latin, abbreviated where no confusion arises.

Math (interpreter + C backend):
  rad(x)        square root (radix)
  pot(x, n)     power (potentia)
  log(x)        natural log
  sin(x)        sine (already Latin)
  cos(x)        cosine (already Latin)
  sors()        random float 0-1 (sors = lot, chance)
  abs(x)        absolute value
  min(a, b)     minimum
  max(a, b)     maximum

String (interpreter + C backend):
  lon(s)            length (longitudo)
  ind(s, n)         character at position n
  tonde(s)          trim whitespace (tonde = clip)
  continet(s, sub)  contains substring
  ini(s, pre)       starts with prefix (incipit)
  des(s, suf)       ends with suffix (desinit)
  iunge(a, b)       concatenate (iunge = join)

String + List (interpreter only — C backend deferred to Phase 11,
because they need series[verba]):
  forma(fmt, ...)   format string with {} placeholders
  scinde(s, sep)    split string into a list
  ordina(list)      sort a list

Implementation notes:
- Math functions emit direct C calls to <math.h> (sqrt, pow, etc.).
- String helpers added to runtime.c: hsd_char_at, hsd_tonde,
  hsd_des, hsd_iunge.
- gen_scribe special-cases boolean-typed arguments so continet/ini/
  des and comparisons print as verum/falsum in the C backend,
  matching the interpreter.
- forma/scinde/ordina in the C backend require series[verba]
  (Phase 11); they raise a clear error there for now.

The higher-order list functions converte (map), elige (filter),
and collige (reduce) are deferred to Phase 16, when first-class
functions exist (they take a function as an argument).

*Demonstrators:* deferred to Phase 11, when scinde works in the C
backend too, so the word-count and grep demonstrators run on both
paths.
*See also:* HSD-architecture-pipeline.md for the full compiler
pipeline, the runtime explanation, and the checklist for adding
built-ins.

---

## Active roadmap

**Phase 10b - UTF-8 support.**
SetConsoleOutputCP(65001) in Windows main bridge. UTF-8 validation
in lexer. Correct verba length semantics (characters vs bytes).
lege reading full Unicode codepoints.
*See also:* HSD-known-issues.md.

**Phase 11 - Polymorphic series + list literals in backend.**
series[verba], series[realis], and [1, 2, 3] literals all compile
in the C backend. This also unblocks forma, scinde, and ordina in
the C backend (deferred from Phase 10).
*Demonstrators:* bubble sort on series[verba]; Caesar cipher;
the Phase 10 word-count tool and grep-like filter (now running on
both interpreter and C backend).

**Phase 11b - Multidimensional arrays (matrices).**
series[series[realis]] and 2D array syntax. Row/column access,
basic matrix operations.
*Unblocks: transformer demonstrator, linear algebra stdlib.*
*Demonstrators:* matrix multiplication; 2D pixel array.

**Phase 12 - File I/O.**
Reading and writing files, working with paths.
*Unblocks: transformer (loading weights), self-hosted lexer.*
*Demonstrators:* cat-like file printer with line numbering;
log filter.

**Phase 12b - MILESTONE: Minimal transformer in HSD.**
A small transformer model written entirely in HSD. No external
libraries, no Python, no PyTorch. Compiled to a standalone C
executable. Inspired by llama.cpp — same philosophy, HSD syntax.

Goals:
- Show HSD viability for numerical and ML-adjacent code
- Understand transformer internals by implementing from scratch
- Demonstrate speed advantage over Python for inference
- Standalone executable, zero dependencies

Target use case: lightweight inference on edge and embedded targets
where Python is too heavy and C is too verbose. Also
privacy-sensitive environments (hospitals, legal, finance) where
data cannot leave the device.

Requires: Phase 10 (math stdlib), Phase 11b (matrices),
Phase 12 (file I/O to load weights).
*Demonstrators:* 1-2 layer attention model; benchmark vs Python.

**Phase 13 - Tuples.**
Grammar, semantic analysis, codegen. Lightweight alternative to
genus for ad-hoc grouping.
*Demonstrators:* divmod (quotient and remainder); min_max (min
and max in one pass).

**Phase 14 - Maps.**
Hash maps in the language and C backend.
*Unblocks: symbol tables in the self-hosted semantic analyzer.*
*Demonstrators:* word-frequency counter; phone book with lookup.

**Phase 15 - Variant types + pattern matching.**
A type can now have alternatives. cum ... est ... destructures them.
*Unblocks: AST as a clean variant type, which makes the
self-hosted lexer, parser, and analyzer much nicer.
Also unblocks Phase 7 (Result[T, E]).*
*Demonstrators:* arithmetic expression evaluator with variant AST;
shape area calculator using variants.

**Phase 7 (completion) - Error handling with Result[T, E].**
Now that variant types exist, Result[T, E] is a real variant type.
Standard library functions become recoverable.

**Phase 16 - First-class functions.**
Function values, closures. Opens the door to higher-order stdlib
(converte/map, elige/filter, collige/reduce over series) and
cleaner visitor patterns.
*Demonstrators:* implement converte, elige, collige for series;
event-handler dispatcher.

**Phase 17 - Visibility.**
publicum / privatum on declarations. Real encapsulation.
*Demonstrators:* stack library with private internals.

**Deep dive: understanding the Rust compiler.**
Before writing the self-hosted components, a study phase: reading
and understanding the existing Rust implementation of each component
in depth. The goal is to understand every design decision well
enough to rewrite it from scratch in HSD, not to translate Rust
mechanically. One session per component, before each milestone.

**Phase 18 - MILESTONE: Lexer in HSD.** ⭐
A complete HSD lexer written in HSD using variant types for tokens.
Validated by diffing output against the Rust lexer on the test corpus.
*Demonstrators:* the lexer itself; keyword-frequency tool.

**Phase 19 - MILESTONE: Parser in HSD.** ⭐
Recursive-descent parser in HSD, building variant-typed AST nodes,
with error recovery.
*Demonstrators:* the parser itself; HSD source pretty-printer.

**Phase 20 - MILESTONE: Semantic analyzer in HSD.** ⭐
Name resolution, type checking, type inference — all in HSD.
Uses maps for symbol tables, variant types for AST visiting.
*Demonstrators:* the analyzer itself.

**Phase 21 - MILESTONE: Codegen in HSD.** ⭐
The C-code generator rewritten in HSD. Reads an AST, emits C.
*Demonstrators:* the codegen itself; AST pretty-printer.

**Phase 22 - Real actor concurrency.**
A scheduler (thread pool, work-stealing), per-actor mailboxes,
asynchronous message delivery. The actor model finally delivers
on its promise.
*Demonstrators:* ping-pong between concurrent actors; producer/
consumer pipeline; parallel word-count with worker actors.

**Phase 23 - MILESTONE: Compiler self-hosting complete.** ⭐
All four compiler components running in HSD. The Rust compiler
becomes the bootstrap compiler. The HSD compiler is in HSD.
*Demonstrators:* HSD compiler compiles its own source; output
identical to Rust-compiled version on the test corpus.

**Phase 24 - LLVM backend.**
Second backend emitting LLVM IR. --backend=llvm vs --backend=c.
C backend stays as development backend.
*Demonstrators:* C vs LLVM benchmark; cross-compilation demo.

**Phase 25 - Foreign Function Interface (FFI).**
Clean way to declare and call functions from external C libraries.
*Unblocks: graphics, networking, ML inference bindings.*
*Demonstrators:* call sqrt from libc; HTTP GET via libcurl.

**Phase 26 - Enhanced nativum and low-level control.**
SIMD intrinsics, manual memory layout, tight loops without ARC.
*Demonstrators:* SIMD vector add; manual memory pool.

**Phase 27 - Math and linear algebra stdlib.**
vec2, vec3, vec4, mat2-mat4, quaternions, transformations.
Built on Phase 26 SIMD.
*Demonstrators:* ray-sphere intersection; 3D camera transform.

**Phase 28 - Networking stdlib.**
TCP/UDP sockets, DNS, synchronous and actor-based async.
*Demonstrators:* echo TCP server; chat client/server pair.

**Phase 29 - HTTP/web framework built on actors.**
Small HTTP server where each request is an actor.
*Demonstrators:* hello-world web server; JSON todo API;
chat server with multiple clients as actors.

**Phase 30 - DataFrame stdlib.**
Tabular data, typed columns, vectorized operations.
*Demonstrators:* mini-Pandas tool (CSV, filter, groupby);
parallel ETL pipeline using actors.

**Phase 31 - Graphics bindings.**
SDL2 (windowing, 2D, input) and OpenGL (3D), via FFI.
*Demonstrators:* bouncing ball; Conway's Game of Life; wireframe
3D cube; Gaussian splat viewer on CPU (loads a .ply file of 3D
Gaussians — serious test of SIMD, math stdlib, and graphics).

**Phase 32 - ML inference bindings.**
FFI bindings to ONNX Runtime and GGUF loaders.
Using ML, not training ML.
*Demonstrators:* image classifier (MobileNet via ONNX); sentiment
analyzer; tiny LLM inference via GGUF (1B parameter model).

**Phase 33 - Tooling.**
Formatter, hsd build/run wrapper, improved REPL, language server.

**Phase 34 - User documentation.**
Language reference, tutorial, worked examples. At this point HSD
is usable by someone outside the project.

**Phase 35 - MILESTONE: Runtime self-hosting — true self-hosting complete.** ⭐
The runtime library (runtime.c today) rewritten in HSD using
nativum blocks. ARC, string handling, actor scheduler, file I/O
— all in HSD. HSD depends on nothing it did not write itself,
modulo OS calls.
*Demonstrators:* ARC retain/release in HSD; hsd_lege in HSD;
identical output on entire test corpus.

**Phase 36 - Hermetic mode (optional, Linux only).**
Direct syscalls in assembly, removing the libc dependency.
--hermetic build produces executables linking no external libraries.
*Demonstrators:* hermetic hello-world (zero libs, verified with ldd).

---

## Notes

**On self-hosting milestone order.** The original roadmap placed
the lexer in HSD at Phase 13, before variant types. This was an
error — a lexer without variant types uses genus with string fields
to simulate token variants, producing code that would need to be
rewritten after Phase 15. All four self-hosting milestones are now
after Phase 15.

**On Phase 7 (error handling).** Deliberately deferred past Phase 15.
Go-style multiple return was evaluated and rejected. Result[T, E]
with real variant types is the right design.

**On the transformer milestone (Phase 12b).** A demonstration
milestone, not a step toward ML production tooling. The value is
showing HSD handles numerical code cleanly, producing a standalone
executable with no dependencies. The use case is lightweight
inference on edge/embedded targets — the llama.cpp philosophy
applied to HSD.

**On GPU computing.** HSD targets CPU. GPU support (CUDA, OpenCL)
is not in this roadmap. The nativum FFI mechanism allows calling
external CUDA libraries — the same approach used by Python
(PyTorch), Rust (cudarc), and C#. Native GPU kernel compilation
is out of scope.

**On the dual backend.** C backend stays as the development backend
(human-readable output) alongside the future LLVM production backend.

**On compiler vs runtime self-hosting.** Phase 23 = compiler in HSD.
Phase 35 = runtime in HSD. Only after Phase 35 does HSD depend on
nothing it did not write itself.

**On the package manager.** Module resolution uses HSD_PATH today.
A future dev tool will manage ~/.hsd/packages/ with hsd.toml as
per-project config. Post-Phase 23 work.
*See also:* HSD-known-issues.md.

**On HSD positioning.** HSD aims at: actor-based web backends,
parallel data pipelines, command-line tools and compilers,
lightweight ML inference on edge targets, technical visualization.
Not targeting: Python data science, JavaScript frontend, GPU
training pipelines. Specialization is more honest than universality.
