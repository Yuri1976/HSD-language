\# HSD — Known Issues and Miscellaneous Debt



This document tracks known limitations, rough edges, and deferred

fixes that do not belong to a specific phase and are not severe enough

to block progress. Think of it as a running list of "we know about

this, it's not forgotten".



For ARC-specific debt, see `HSD-arc-debt.md` — that document stands

alone because the ARC implementation is complex enough to warrant

its own accounting.



\---



\## Active issues



\### UTF-8 support in the runtime



\*\*Severity\*\*: low (does not affect correctness, only display)

\*\*Affects\*\*: C backend on Windows



The generated C code outputs strings using `printf`. On Windows, the

console defaults to CP1252 or CP850, which cannot render non-ASCII

UTF-8 characters. Multi-byte characters like the em dash (U+2014)

are mangled into garbage sequences such as `ΓÇö`.



\*\*Workaround\*\*: run `chcp 65001` in the terminal before executing

a compiled HSD program, or avoid non-ASCII characters in string

literals until this is properly fixed.



\*\*Planned fix\*\*: emit `SetConsoleOutputCP(65001)` inside the `main`

bridge in the C backend, guarded by `#ifdef \_WIN32`. This is a

one-line fix in `codegen.rs` (`generate()`, the `main` bridge

section). The deeper issue — UTF-8 validation in the lexer, correct

`verba` length semantics (characters vs bytes), and `lege` reading

full Unicode codepoints — requires a dedicated phase.



\*\*Deferred to\*\*: Phase 10b (UTF-8), scheduled right after the stdlib

bootstrap (Phase 10) when string handling is being formalized anyway.



\---



\## Resolved issues



\*(none yet — this section will grow as issues above are fixed)\*



\---



\## How to use this document



When a new rough edge is discovered during development, add it here

with:

\- a short title

\- severity (low / medium / high)

\- what is affected

\- a clear description of the symptom

\- a known or suspected fix

\- what it is deferred to (a phase, or "unknown")



When an issue is fixed, move it to the Resolved section with a note

on what phase or commit addressed it.

