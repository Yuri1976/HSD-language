# HSD — Known Issues and Roadmap

This document tracks what is currently broken or unfinished in HSD,
what stands between the project today and HSD as a real, usable
programming language, and the order in which these pieces will be
tackled. The sequence is not time-based — there are no dates. It is a
dependency-driven order: each phase delivers something the next phases
can rely on.

**The project's North Star: true self-hosting.** A language proves
itself real when it can rebuild itself from its own source — both the
compiler and the runtime written in the language itself. The roadmap
is organized around that goal. Compiler self-hosting (Phase 22)
arrives first; runtime self-hosting (Phase 34) closes the circle.

**Self-hosting is incremental.** Not as a final rewrite, but as five
milestones interleaved into the language's growth: four for the
compiler (lexer, codegen, semantic analyzer, parser), one for the
runtime. Each one validates that HSD has become rich enough to
express that part of itself.

**Beyond self-hosting, into application domains.** Phases 23–33 add
the capabilities HSD needs to be applied to real problems: an FFI, a
production backend (LLVM), low-level control for performance, vector
and matrix math, networking, a web framework, a DataFrame library,
graphics bindings, and ML inference bindings. These turn HSD from
"a language that exists" into "a language you can build things with".

**Demonstrators.** Each phase lists one or more small programs
(50–200 lines) that exercise the new features. These validate that
the phase is genuinely complete and form a growing gallery of
examples for anyone learning HSD.

---

## Known Issues

The current state, grouped by where the issues live.

### Memory
- **ARC is designed but not implemented.** The C backend leaks every
  heap allocation until program exit. Small programs work; long-running
  ones accumulate memory indefinitely. *Phase 6 in progress.*

### Type system
- **`genus` (record types) is grammar-only.** The parser accepts
  records and the semantic analyzer verifies their fields, but neither
  the interpreter nor the C backend executes them.
- **No variant types.** No way to express "a value is one of several
  shapes, each with its own data" — what Rust calls `enum` with data,
  what an AST is naturally written as. Without this, structured data
  is awkward.
- **`series[T]` is partially polymorphic.** The semantic analyzer
  accepts `series[verba]`, `series[realis]`, etc., but the C backend
  only emits code for `series[numerus]`.
- **List literals (`[1, 2, 3]`) are interpreter-only.** The C backend
  does not parse or emit them.

### Concurrency
- **Actors are synchronous.** `mitte` invokes the handler on the same
  thread, immediately. There is no mailbox, no scheduler, no
  parallelism. The actor syntax works but the safety guarantee it
  implies (no shared memory, no races) is vacuous — there is no
  concurrency to race in yet.

### Control flow
- **No error handling.** No `try`/`catch`, no `Result<T, E>`, no
  exceptions. Runtime errors (like `numerus_ex("hello")`) terminate
  the program. There is no way to recover. *Phase 7 in progress.*
- **No pattern matching.** No `match`/`case`. Visiting structured data
  (especially future variant types or ASTs) is verbose without it.

### Modules
- **`affer` is a placeholder.** The keyword exists but module loading
  and namespace separation are not implemented. All HSD code must
  live in a single file.

### Standard library
- **Minimal.** Currently: `scribe`, `lege`, `numera`, `numerus_ex`,
  `realis_ex`. Missing: string operations, list operations beyond
  iteration, math beyond arithmetic operators, file I/O, time,
  environment, networking, vector/matrix math, tabular data
  structures, bindings to external libraries.

### Backend
- **Single backend.** Only the C backend exists. No LLVM, no
  alternative path. The C backend is good for development (legible
  output, easy to debug) but ties HSD to having an external C
  compiler installed.

### Runtime
- **Runtime is in C.** The runtime library (memory management,
  string I/O, actor support) is hand-written C. Until the runtime is
  rewritten in HSD, the language is only *partially* self-hosting:
  the compiler can be in HSD, but its outputs still depend on C
  code that HSD did not produce.

### Foreign code
- **No FFI.** HSD has no clean way to declare and call functions from
  external C libraries. Without this, HSD applications cannot reach
  the existing ecosystem (graphics libraries, networking libraries,
  databases, ML runtimes, anything that lives in C).

### Tooling
- **None beyond the compiler.** No formatter, no language server, no
  REPL with edit support, no build tool beyond `cargo run -- build`,
  no package manager, no debugger integration.

---

## Priority Tiers

A categorical view: how much each missing piece blocks.

### Tier 1 — Foundation
Without these, HSD cannot honestly call itself complete.
- ARC
- `genus` end-to-end
- Error handling (minimal)

### Tier 2 — Language substance
With these, HSD becomes a language someone could write a real program
in — including its own compiler.
- Module system
- Standard library bootstrap (strings, math, list operations)
- Polymorphic `series` and list literals in backend
- File I/O
- Tuples
- Variant types + pattern matching
- First-class functions and closures
- Maps
- Visibility (`publicum` / `privatum`)

### Tier 3 — Compiler self-hosting milestones
Each one is the rewrite of one compiler component from Rust into HSD,
validated by running both versions on the same input and comparing
outputs. After all four, **the compiler is in HSD** — but the
runtime is still in C.
- Lexer in HSD
- Codegen in HSD
- Semantic analyzer in HSD
- Parser in HSD — *compiler self-hosting complete*

### Tier 4 — Signature feature
The actor model delivering on its promise.
- Real actor concurrency (mailbox, scheduler, parallelism)

### Tier 5 — Production backend and low-level control
- LLVM backend (kept alongside C backend until the language is fully
  stable; the C backend stays useful as a debugging path)
- Foreign Function Interface (FFI) to C libraries
- Enhanced `nativum` blocks for performance-critical code (SIMD,
  manual memory layout, tight loops without ARC overhead)

### Tier 6 — Application domains and ecosystem
The libraries and bindings that let HSD reach real use cases.
- Math and linear algebra stdlib (vectors, matrices, transformations)
- Networking stdlib (TCP/UDP sockets, DNS)
- Web/HTTP framework built on actors
- DataFrame stdlib (tabular data, ETL, aggregations)
- Graphics bindings (SDL2, OpenGL, optionally a small 2D/3D wrapper)
- ML inference bindings (ONNX Runtime, GGUF — *using* pre-trained
  models, not training new ones)

### Tier 7 — Polish
What turns a usable language into one others would adopt.
- Tooling (formatter, build tool, REPL, language server)
- User documentation (tutorial, language reference)

### Tier 8 — True self-hosting
The final step. The runtime rewritten in HSD; the language depending
on nothing but a small irreducible core (optionally direct syscalls
on Linux, libSystem on macOS, ntdll on Windows).
- Runtime self-hosting (the runtime library written in HSD with
  `nativum` blocks where necessary)
- Hermetic mode (optional): direct syscalls in assembly for Linux,
  removing the libc dependency

---

## Sequential Roadmap

The order of work, in dependency-respecting phases. Numbering
continues from Phase 5c, the most recent completed phase.
Self-hosting milestones are marked **⭐**.

**Phase 6 — ARC** *(in progress)***.** Reference counting for all
heap-allocated values. Retain/release inserted by the C backend at
the appropriate places. Cycle detection deferred (cycles are rare in
HSD's value-oriented design). *Unblocks: everything downstream — no
real program can run without solid memory management.*
*Demonstrators:* stress test — create and discard 10,000 strings,
validate memory returns to baseline; build and tear down a one-million
node linked list.

**Phase 7 — Error handling** *(in progress)***.** Minimal first
pass. Decide between Go-style multiple return
(`munus f() -> numerus, error`) and a `Result` type before starting.
Standard-library functions like `numerus_ex` become recoverable.
*Unblocks: writing a compiler that reports errors instead of crashing.*
*Demonstrators:* a calculator REPL that handles invalid input
gracefully; a number-guessing game with proper input validation.

**Phase 8 — `genus` end-to-end.** Record types fully working in
interpreter and C backend. Field access, construction, equality. The
first composite user-defined type. *Unblocks: representing structured
data — tokens, AST nodes, symbol-table entries.*
*Demonstrators:* a small address book (Person records, list of them,
print formatted); a basic geometry library (Point, Circle, Rectangle
with area/perimeter).

**Phase 9 — Module system.** A working `affer`, namespace separation,
multi-file compilation. *Unblocks: splitting both the standard
library and any future self-hosted compiler across files.*
*Demonstrators:* split the geometry library from Phase 8 across
multiple files; a small math library imported by a program that uses
it.

**Phase 10 — Standard library bootstrap.** First real stdlib, written
in HSD where possible (with `nativum` blocks where needed). Focused
on what a compiler needs: string operations (split, trim, indexing,
slicing, concatenation, formatting), list operations
(map/filter/reduce/sort), math (sqrt, pow, log, sin/cos, random).
*Demonstrators:* a word-count tool (reads stdin, splits into words,
counts and prints); a simple `grep`-like text filter.

**Phase 11 — Polymorphic series + list literals in backend.** Bring
the C backend up to interpreter parity for collection types.
`series[verba]`, `series[realis]`, and `[1, 2, 3]` literals all
compile. *Demonstrators:* a sorting playground (bubble sort,
insertion sort on `series[verba]`); a Caesar cipher that works on
strings.

**Phase 12 — File I/O.** Reading and writing files, working with
paths, the basics of the filesystem. Moved up because the lexer
milestone needs it. *Unblocks: the first compiler self-hosting
milestone.* *Demonstrators:* a `cat`-like file printer with line
numbering; a simple log filter that reads a file and prints lines
matching a pattern.

**⭐ Phase 13 — MILESTONE: Lexer in HSD.** A complete HSD lexer
written in HSD: reads a source file, produces a token stream, reports
errors with file and column information. Validated by diffing its
output against the Rust lexer on the test corpus. *Demonstrators:*
the lexer itself; a small companion tool that counts keyword
frequencies in HSD source files.

**Phase 14 — Tuples.** Grammar, semantic analysis, codegen.
Lightweight alternative to `genus` for ad-hoc grouping. Useful for
multiple returns if Phase 7 went the Go route.
*Demonstrators:* `divmod` (returns quotient and remainder as a
tuple); `min_max` (returns min and max of a list in one pass).

**Phase 15 — Variant types + pattern matching.** Treated as a single
phase because they belong together. A `genus` can now have
alternatives (`Cerchio(r: realis) | Rettangolo(w: realis, h:
realis)`), and `cum ... est ...` (or similar Latin syntax — to
decide) destructures them. *Unblocks: AST as a clean variant type,
which makes the parser and analyzer milestones much nicer.*
*Demonstrators:* a tiny arithmetic expression evaluator (variant AST:
number, plus, minus, times, divide); a shape area calculator using
variants.

**Phase 16 — Maps.** Hash maps in the language and the C backend.
*Unblocks: symbol tables in the self-hosted semantic analyzer.*
*Demonstrators:* a word-frequency counter (using a map instead of
sorted lists); a simple phone book with lookup by name.

**⭐ Phase 17 — MILESTONE: Codegen in HSD.** The C-code generator
rewritten in HSD. Reads a serialized AST (the Rust version emits
it), walks it, produces C code. *Codegen is a good second milestone
— mechanical work that stresses string building and recursion
without needing the most advanced language features.*
*Demonstrators:* the codegen itself; a small companion that takes a
JSON-encoded AST and emits both C and a pretty-printed form.

**Phase 18 — First-class functions.** Function values, closures.
Opens the door to higher-order stdlib functions (`map`, `filter`,
`reduce` over `series`) and to cleaner visitor patterns in the
self-hosted analyzer. *Demonstrators:* implement `map`, `filter`,
`reduce` for `series` and exercise them on a small dataset; a tiny
event-handler dispatcher.

**Phase 19 — Visibility.** `publicum` / `privatum` on declarations.
Real encapsulation. *Demonstrators:* a small stack library exposing
only push/pop/peek while keeping internal state private; refactor
an earlier demonstrator to hide its helpers.

**⭐ Phase 20 — MILESTONE: Semantic analyzer in HSD.** Name
resolution, type checking, type inference — all rewritten in HSD.
Uses maps for symbol tables, variant types for AST visiting,
first-class functions for the visitor pattern. *Demonstrators:* the
analyzer itself; a small type-checker for a toy expression language.

**Phase 21 — Real actor concurrency.** A scheduler (pool of OS
threads, work-stealing), per-actor mailboxes, asynchronous message
delivery. The actor model finally delivers on its promise. *This is
where HSD's distinctive feature actually starts distinguishing it.*
*Demonstrators:* ping-pong between two actors running concurrently;
a producer/consumer pipeline with a queue actor; a parallel
word-count that distributes chunks across worker actors.

**⭐ Phase 22 — MILESTONE: Parser in HSD — compiler self-hosting
complete.** Recursive-descent parser in HSD, building variant-typed
AST nodes, with proper error recovery. Once this lands, the Rust
compiler is the *bootstrap compiler*: needed once to compile the
HSD-written compiler, then optional. **The HSD compiler is in
HSD.** *Demonstrators:* the parser itself; the canonical proof —
the HSD compiler compiles its own source, and the output is
identical to the Rust-compiled version on the test corpus.

**Phase 23 — LLVM backend.** A second backend that emits LLVM IR
instead of C, used via the `inkwell` Rust crate (or its HSD
equivalent if self-hosting has progressed enough). Selected via
`--backend=llvm`; the C backend stays as `--backend=c` and remains
the default for development (legible output) until the language is
fully stable. *Demonstrators:* a benchmark suite comparing C and
LLVM backends on identical programs; a cross-compilation demo
(compile for ARM from x86, or for WebAssembly).

**Phase 24 — Foreign Function Interface (FFI).** A clean way to
declare and call functions living in external C libraries. Syntax
for declaring foreign signatures, automatic ARC handling across the
boundary, conversion of HSD types to C ABI. *Unblocks: all
real-world libraries — graphics, networking, databases, ML
runtimes.* *Demonstrators:* call `printf` and `sqrt` from libc; bind
to a small external C library (libcurl for a one-line HTTP GET
tool).

**Phase 25 — Enhanced `nativum` and low-level control.** Make
`nativum` blocks suitable for performance-critical code: SIMD
intrinsics access (process 4–8 floats in parallel), manual memory
layout control (packed structs, alignment hints, struct-of-arrays
patterns), tight loops without ARC overhead. *Unblocks: graphics
and numerical code that needs raw speed.* *Demonstrators:* a
SIMD-accelerated vector add (compare timing against scalar version);
a manual memory pool for fixed-size objects.

**Phase 26 — Math and linear algebra stdlib.** Vectors (`vec2`,
`vec3`, `vec4`), matrices (`mat2`, `mat3`, `mat4`), quaternions,
transformations (translate, rotate, scale, project). Built on top
of the SIMD primitives from Phase 25 where beneficial. *Unblocks:
graphics, physics, geometry, anything 3D, plus the math
foundations for DataFrames and ML inference.* *Demonstrators:* a
ray-sphere intersection test; a 3D camera transformation calculator
that takes view parameters and outputs a view-projection matrix.

**Phase 27 — Networking stdlib.** TCP and UDP sockets, basic DNS
resolution, IP address handling. Listening, accepting, reading,
writing — both synchronous and (using Phase 21's actor scheduler)
asynchronous. *Demonstrators:* an echo TCP server that handles
multiple connections concurrently; a chat client/server pair using
raw text protocol.

**Phase 28 — HTTP/web framework built on actors.** A small HTTP
server library where each request is handled by an actor, with
typed routes and JSON serialization in the stdlib. *This is where
HSD's actor model gets its most natural showcase — every request,
every client, every background task is an actor.*
*Demonstrators:* a "hello world" web server; a JSON API for a
simple todo list (CRUD endpoints); a chat server with multiple
connected clients, each connection as an actor.

**Phase 29 — DataFrame stdlib.** A tabular data structure with
typed columns and vectorized operations (`filter`, `map`,
`groupby`, `join`, `aggregate`). Built on top of the math stdlib
(Phase 26). The Rust crate Polars is the modern reference design.
*Unblocks: data engineering, ETL, analytics.* *Demonstrators:* a
mini-Pandas tool (load CSV, filter, groupby, print); a parallel
ETL pipeline using actors (one actor per stage); a statistical
summary tool (mean, median, percentiles for a CSV column).

**Phase 30 — Graphics bindings.** Bindings to SDL2 (for
windowing, 2D drawing, input handling) and OpenGL (for 3D
rendering). Built on Phase 24's FFI and Phase 26's math stdlib.
*Demonstrators:* a bouncing ball in a window; Conway's Game of
Life with graphical display; a wireframe 3D cube viewer with mouse
rotation; **a Gaussian splat viewer on CPU** — loads a `.ply` file
of 3D Gaussians and renders them to a window (a serious test of
Phase 25's SIMD, Phase 26's math, and Phase 30's graphics, and a
demo that connects HSD to real XR/3D capture work).

**Phase 31 — ML inference bindings.** FFI bindings to ONNX Runtime
and GGUF loaders, allowing HSD programs to load and run pre-trained
models. *Using ML, not training ML.* Training serious models
requires CUDA/ROCm bindings and an ecosystem that HSD cannot
realistically reproduce on its own — those bindings, if they ever
come, are work for a future community, not for the founder.
*Demonstrators:* an image classifier using a pre-trained ONNX
model (e.g., MobileNet); a sentiment analyzer using a small ONNX
text model; a tiny LLM running inference via GGUF (e.g., a
1B-parameter model for short text completion).

**Phase 32 — Tooling.** A formatter (canonical layout for HSD
code). A `hsd build` / `hsd run` command wrapping the compiler.
An improved REPL with line editing. Later: a language server for
editor integration. *Demonstrators:* the tools themselves; a CI
configuration using them to enforce style on a sample project.

**Phase 33 — User documentation.** Language reference (formal),
tutorial (informal), worked examples. At this point HSD is usable
by someone outside the project. *Demonstrators:* a "Learn HSD in
Y minutes" page covering the core language in a single screen; a
small book-length tutorial walking through real programs.

**⭐ Phase 34 — MILESTONE: Runtime self-hosting — true self-hosting
complete.** The runtime library (`runtime.c` today) rewritten in
HSD, using `nativum` blocks where the bottom must touch the
operating system. ARC implementation, string handling, actor
scheduler, file I/O wrappers — all in HSD. *This is the milestone
the project crosses to be fully self-hosting: both the compiler
(Phase 22) and the runtime are now in HSD.* The dependency on C
code that HSD did not produce shrinks to whatever is needed to
talk to the operating system — by default a small set of libc
calls. *Demonstrators:* rewrite ARC retain/release in HSD with
`nativum` blocks for the atomic primitives; rewrite `hsd_lege` in
HSD; show that programs compiled with the HSD-written runtime
produce output identical to those compiled with the C runtime, on
the entire test corpus.

**Phase 35 — Hermetic mode (optional).** Replace the libc
dependency on Linux with direct syscalls written in assembly.
A `--hermetic` build mode produces executables that link no
external libraries: HSD talks directly to the kernel. macOS and
Windows do not allow this purely (libSystem and ntdll are
unavoidable), so this mode is Linux-only by design.
*Demonstrators:* a hermetic hello-world that statically links
zero libraries (verified with `ldd`); a hermetic `cat` clone
that works without libc; a benchmark comparing startup time and
binary size between hermetic and standard builds.

---

## Notes

This roadmap is a living document. Phases may be reordered if
implementation work reveals different dependencies. New issues
discovered along the way go into Known Issues; completed phases are
moved out of the roadmap and into the project overview's "Fully
implemented" section.

**On the dual backend.** Keeping C and LLVM backends in parallel is
intentional. The C backend produces output a human can read and
debug; this is invaluable while the language is still evolving and
bugs can be in the codegen as easily as anywhere else. LLVM will
become the production backend, but the C backend stays as the
development backend until HSD is genuinely stable. The point at
which the C backend can be deprecated is not a date — it is when
language work is no longer revealing changes that need debugging at
the codegen level.

**On compiler vs runtime self-hosting.** A common mistake is calling
a language "self-hosting" once its compiler is written in itself.
But the compiler's output still depends on a runtime — the small
library of support functions that every compiled program links
against. HSD distinguishes two milestones: *compiler self-hosting*
(Phase 22) — the compiler is in HSD — and *true self-hosting*
(Phase 34) — the runtime is in HSD too. Only after Phase 34 does
HSD depend on nothing it did not write itself, modulo whatever is
needed to talk to the operating system (libc by default, or direct
syscalls in hermetic mode on Linux).

**On self-hosting milestones.** Each ⭐ milestone is a *parallel
implementation*, not a replacement. The Rust compiler keeps working
throughout. The HSD-written component is a separate program whose
output is diffed against the Rust version on a corpus of test
inputs. Only after Phase 22 (parser in HSD) is the Rust compiler
retired as the active code path; only after Phase 34 (runtime in
HSD) is the C runtime retired. Both retired versions remain in the
repository as the *bootstrap chain*, used once per clean build to
produce a self-hosted HSD.

**On HSD's positioning.** HSD is not trying to compete with
everything. It aims to be coherent with itself and to be very good
at a specific cluster of use cases: actor-based web back-ends
(Phase 28), parallel data pipelines (Phase 29), command-line tools
and compilers (post Phase 22), technical and scientific
visualization (Phase 30, including demos like the Gaussian splat
viewer), and ML inference using pre-trained models (Phase 31).
Training large models, GPU-accelerated rendering pipelines, and
browser-side front-end web are not HSD's targets and likely never
will be. Specialization is more honest than universality, and the
chosen specialization — concurrent, native, statically typed,
actor-first — is one the mainstream does not occupy well today.

**Out of scope.** The opt-in strictness levels (described in the
project overview's long-term vision) are not in this roadmap. They
are a research direction, not a feature with a clear implementation
plan. Likewise, Latin inflection as a metaprogramming layer remains
future exploration. ML training (as opposed to inference) is
deferred to potential future community contributions and is not in
the founder's path.
