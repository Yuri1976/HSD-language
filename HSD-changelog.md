# HSD — Changelog & Notes

This document tracks incremental fixes, tooling improvements, and
small discoveries that don't belong in `HSD-roadmap.md` — the roadmap
tracks phase completion, not the everyday debugging and polish that
happens while building and using the language. Entries are grouped by
theme, most recent first within each group.

This is a working log, not a formal release changelog. Dates are
approximate (session-based, not commit-based).

---

## Lexer

### Tab indentation support — June 2026
The lexer originally rejected tab characters in leading indentation
with an explicit error ("use spaces, not tabs"). Tabs are now accepted
and converted to 4 spaces each before indentation level comparison.
This matters because many editors on Windows default to tabs, and
forcing spaces-only made fast iteration painful.

### CRLF line endings — June 2026
Source files saved on Windows (CRLF line endings, `\r\n`) caused a
lexical error on the very first line ("unexpected character"), because
the `\r` was being read as part of the token stream. Fixed in
`main.rs` by stripping `\r\n` → `\n` immediately after reading the
source file, before it reaches the lexer.

### Dead `main` function removed — June 2026
`lexer.rs` originally contained a standalone `fn main()` left over from
when the lexer was built and tested as an independent program. Once
the lexer became a module used by the rest of the compiler, this `main`
was never called, producing a `dead_code` warning on every build. The
function — and the now-unused `std::env`, `std::fs`, `std::process`
imports it required — have been removed.

---

## C backend (codegen)

### `long` vs `long long` on Windows/MSVC — June 2026
The C backend was emitting `long` for all integer values (`numerus`).
On Linux, `long` is 64-bit, so this worked silently. On Windows with
MSVC, `long` is **32-bit** — any HSD program using integers larger
than ~2.1 billion would silently overflow when compiled, while the
interpreter (which uses Rust's `i64` directly) gave the correct
answer. This produced a visible bug: Project Euler #3
(`600851475143`, well over 32-bit range) returned `0` from the
compiled binary while the interpreter returned the correct `6857`.

Fixed by changing every `long` emission in `codegen.rs` to
`long long`, and the corresponding printf format specifier from
`%ld` to `%lld`. This is a strong example of why testing both
interpreter and compiled paths matters — the bug was invisible until
a real workload with large numbers exposed it.

**Takeaway for the future:** any new codegen path that emits a fixed
C integer type should default to `long long` (or an explicitly-sized
type like `int64_t`), never bare `long`, given the platform's MSVC
target.

---

## Tooling

### `hsd-build.ps1` — build/run/benchmark helper script — June 2026
A PowerShell script was added to the repo root to remove the manual,
repetitive steps around testing an HSD program: switching to
Developer PowerShell for `cl`, running `cargo run build`, then `cl`
by hand, then the `.exe`, then cleaning up stray `.obj` files left in
the repo root.

What it does:
- **Finds the `.hsd` file anywhere in the repo** by name (no need to
  remember or type the full path), and warns if multiple files share
  the same name.
- **Strips a trailing `.hsd`** if accidentally included in the
  `-File` argument.
- **Loads the MSVC developer environment automatically** (via
  `Enter-VsDevShell`) only if `cl` isn't already on the PATH in the
  current shell — so it works from a plain PowerShell window without
  ever needing to open Developer PowerShell manually.
- **Keeps intermediate `.obj` files out of the repo** entirely, by
  redirecting them to `%TEMP%\hsd_obj\` via `cl /Fo:`.
- **Skips recompilation** if the existing `.exe` is newer than the
  `.hsd` source (timestamp check), unless `-Force` is passed — useful
  when re-running the same binary for a benchmark without
  re-triggering the full build → compile pipeline each time.
- **Optional `-Benchmark` mode** with `-Runs N`: runs a warm-up
  execution first (shown but excluded from the average — the first
  run is consistently slower due to cold filesystem cache, antivirus
  scan, and page mapping, not representative of real execution
  speed), then times `N` further runs with `Measure-Command` and
  reports each one plus the average.

Usage:
```powershell
.\hsd-build.ps1 -File Euler05                          # asks interpreter or compiled
.\hsd-build.ps1 -File Euler05 -Modo build               # compiled only
.\hsd-build.ps1 -File Euler05 -Modo both -Benchmark -Runs 3   # timed comparison
```

**Why this matters beyond convenience:** consistent, repeatable
benchmarking is part of the project's plan to track HSD's performance
over time as the compiler matures (see the benchmarks/ folder, planned
but not yet created). Doing this by hand each time invites
inconsistent methodology; the script enforces the same warm-up/measure
pattern every time.

---

## Benchmarking notes

### Cold-start effects are real and consistent — June 2026
Across both the interpreter and the compiled binary, the first
execution in any sequence is reliably slower than subsequent ones —
observed on Project Euler #5 as a jump from ~1.09s down to a stable
~0.140s for the compiled binary after the first run. This is a
Windows/filesystem/antivirus effect, not a property of the program
itself, and is now handled systematically by the benchmark script's
warm-up-then-measure pattern rather than discarded ad hoc.

### Workload size matters for meaningful comparisons — June 2026
Benchmarking a very light program (e.g. Euler #1, which finishes in
tens of milliseconds either way) mostly measures fixed process
startup overhead, not real execution-speed differences between the
interpreter and the compiled binary. Meaningful interpreter-vs-compiled
comparisons need a workload heavy enough (millions of iterations,
as in Euler #5) that the actual execution time dominates over fixed
overhead.
