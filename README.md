# HSD — Hic Sunt Dracones

> *Hic Sunt Dracones* — "here be dragons", the Latin phrase that
> medieval cartographers wrote across the unmapped regions of their
> maps.

HSD is a small, statically typed compiled programming language with
Latin keywords and native actor-based concurrency. It reads like
Python, runs like C, and treats actors as a first-class language
construct rather than a library feature.

This is a personal project of mine, started over five years ago in C,
recently revived and rewritten in Rust. It runs end-to-end today —
source files lex, parse, type-check, and either execute through a
tree-walking interpreter or compile to native executables through a
C backend. Several pieces of the original design (automatic memory
management, true actor concurrency) are still on paper. The rest
works.

---

## A taste of the language

```romanes
munus factorial(n: numerus) -> numerus
    si n <= 1
        redde 1
    aliter
        redde n * factorial(n - 1)

munus principale() -> nihil
    per i in numera(1, 6)
        scribe("factorial(", i, ") = ", factorial(i))
```

Output:

```
factorial(1) = 1
factorial(2) = 2
factorial(3) = 6
factorial(4) = 24
factorial(5) = 120
```

Latin keywords, Python-style indentation, Rust-style type
annotations on function signatures with inference inside.

---

## Setting up

### Requirements

- **Rust toolchain** (1.70 or newer). Install from
  [rustup.rs](https://rustup.rs/) if you do not have it.
- **A C compiler**, needed for the backend to produce executables:
  - Linux: `gcc` (usually pre-installed) or `clang`
  - macOS: `clang` (comes with Xcode Command Line Tools)
  - Windows: **Microsoft Visual C++ (`cl`)**, included with
    [Visual Studio](https://visualstudio.microsoft.com/) or the
    standalone Build Tools. The HSD tooling (including
    `hsd-build.ps1`) targets `cl` specifically — MinGW is not
    tested and may require manual adjustments to the build commands.

  On Windows, `cl` is available in the **Developer PowerShell for VS**
  or **Developer Command Prompt for VS** that Visual Studio installs.
  The `hsd-build.ps1` helper script loads the MSVC environment
  automatically, so you do not need to open those manually when using
  the script.

### Building HSD

Clone the repository and build the compiler:

```sh
git clone https://github.com/Yuri1976/HSD---Hic-Sunt-Dracones---PUBLIC.git
cd HSD---Hic-Sunt-Dracones---PUBLIC
cargo build --release
```

The compiler binary will be in `target/release/`.

### Running an example through the interpreter

The fastest way to try HSD: pass a source file to `cargo run`, with
no extra arguments, and the interpreter executes it directly.

```sh
cargo run -- examples/factorial.hsd
```

### Compiling to a native executable

Use the `build` subcommand to translate HSD to C, then compile it
with your platform's C compiler.

**On Linux or macOS:**

```sh
cargo run -- build examples/factorial.hsd
gcc examples/factorial.c runtime/runtime.c -I runtime -o factorial
./factorial
```

**On Windows (Developer PowerShell for VS):**

```powershell
cargo run -- build examples\factorial.hsd
cl examples\factorial.c runtime\runtime.c /I runtime
.\factorial.exe
```

The `build` step produces a `.c` file next to the source. The C
compiler then links it with the HSD runtime library to produce a
standalone native executable.

### `hsd-build.ps1` — Windows helper script

On Windows, the full build cycle (generate C, compile with `cl`, run)
involves several steps and requires the MSVC developer environment to
be active. The included `hsd-build.ps1` script automates all of this
from a plain PowerShell window, with no need to switch to Developer
PowerShell manually.

```powershell
# First time only: allow local scripts to run
Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
Unblock-File -Path .\hsd-build.ps1

# Run (asks interactively: interpreter or compiled)
.\hsd-build.ps1 -File Euler05

# Compiled only, with timed benchmark over 3 runs (warm-up excluded)
.\hsd-build.ps1 -File Euler05 -Modo build -Benchmark -Runs 3

# Both interpreter and compiled, side-by-side timing comparison
.\hsd-build.ps1 -File Euler05 -Modo both -Benchmark -Runs 3
```

The script finds `.hsd` files by name anywhere in the repo, loads the
MSVC environment automatically if needed, keeps intermediate `.obj`
files out of the repo (redirected to `%TEMP%\hsd_obj\`), skips
recompilation when the binary is already up to date, and optionally
benchmarks execution with a warm-up run excluded from the reported
average.

---

## Project status

HSD is in active development. The compiler works, the language runs,
but several foundational features are still being implemented. The
[roadmap](HSD-roadmap.md) tracks what is done, what is in progress,
and what is planned.

Current state in brief:
- Lexer, parser, semantic analyzer, interpreter, and C backend
  all working end-to-end.
- Phase 6 (ARC — automatic reference counting) is in progress. Until
  it lands, the C backend leaks heap memory; small programs are
  fine, long-running ones accumulate.
- Phase 7 (error handling) is in progress.

This is a project in motion, not a finished tool. Expect rough edges.

---

## Documentation

The four documents in this repository cover different aspects of HSD.
Each one is meant to stand on its own.

- **[HSD-project-overview.md](HSD-project-overview.md)** — the *what*.
  The project's design philosophy, language at a glance, compiler
  architecture, implementation status, long-term vision, and the
  languages and books that shaped HSD.

- **[HSD-grammar-EBNF.md](HSD-grammar-EBNF.md)** — the *how to
  write it*. The formal grammar in EBNF notation, the reference
  document for parsing.

- **[HSD-language-comparison.md](HSD-language-comparison.md)** — the
  *where it stands*. A side-by-side comparison with Python, C, and
  Rust, with worked examples (factorial, basic and interactive) and
  systematic tables across paradigms, types, vocabularies, and
  features.

- **[HSD-roadmap.md](HSD-roadmap.md)** — the *where it's going*.
  Known issues, priority tiers, the sequential roadmap with 35+
  numbered phases, self-hosting milestones, and small demonstrator
  programs for each phase.

- **[HSD-memory-model.md](HSD-memory-model.md)** — the *how it
  thinks about memory*. A standalone explanation of HSD's memory
  management strategy (ARC), why it was chosen, where its limits
  are, and how those limits will be addressed.

- **[HSD-changelog.md](HSD-changelog.md)** — the *what changed*.
  Incremental fixes, tooling improvements, and engineering notes
  that don't belong in the roadmap (which tracks phase completion,
  not day-to-day progress). Useful context for understanding why
  certain decisions were made.

If you only have time for one document, start with the project
overview. If you are curious about the technical decisions, the
memory model document is the deepest dive available.

---

## Examples

The `examples/` directory contains small programs that exercise
specific language features:

- `factorial.hsd` — recursion, integer arithmetic, the `per` loop
- `09_actor.hsd` — an actor with state and message handlers
- `10_input.hsd` — reading from standard input, parsing
- `11_lista.hsd` — list construction, iteration, summation
- `08_functions.hsd` — function definitions and calls

The `examples/ProjectEuler/` subdirectory contains solutions to
Project Euler problems written in HSD, verified on both the
interpreter and compiled binary:

- `Euler01.hsd` — sum of multiples of 3 and 5 below 1000
- `Euler02.hsd` — sum of even Fibonacci numbers below 4 million
- `Euler03.hsd` — largest prime factor of 600851475143
- `Euler04.hsd` — largest palindrome product of two 3-digit numbers
- `Euler05.hsd` — smallest number divisible by all integers 1 to 20

`HSD-euler05-benchmark.md` documents a three-way performance
comparison (HSD interpreter, HSD compiled to C, Python 3.14) on
Euler #5, with methodology and raw numbers.

Each of these can be run through the interpreter or compiled
through the C backend using the commands above.

---

## Contributing

HSD is a personal project and not yet structured for outside
contributions. The roadmap is the founder's path; pull requests
that align with it are welcome but not solicited. If you have
questions, suggestions, or want to discuss design choices, opening
an issue is the right place.

---

## License

HSD is released under the MIT License. See
[LICENSE](LICENSE) for the full text.

---

*The dragons are a little more mapped than they were five years
ago, but the map is far from finished.*
