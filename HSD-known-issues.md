# HSD — Known Issues and Miscellaneous Debt

This document tracks known limitations, rough edges, and deferred
fixes that do not belong to a specific phase and are not severe enough
to block progress. Think of it as a running list of "we know about
this, it's not forgotten".

For ARC-specific debt, see `HSD-arc-debt.md` — that document stands
alone because the ARC implementation is complex enough to warrant
its own accounting.

---

## Active issues

### UTF-8 support in the runtime

**Severity**: low (does not affect correctness, only display)
**Affects**: C backend on Windows

The generated C code outputs strings using `printf`. On Windows, the
console defaults to CP1252 or CP850, which cannot render non-ASCII
UTF-8 characters. Multi-byte characters like the em dash (U+2014)
are mangled into garbage sequences such as `ΓÇö`.

**Workaround**: run `chcp 65001` in the terminal before executing
a compiled HSD program, or avoid non-ASCII characters in string
literals until this is properly fixed.

**Planned fix**: emit `SetConsoleOutputCP(65001)` inside the `main`
bridge in the C backend, guarded by `#ifdef _WIN32`. This is a
one-line fix in `codegen.rs` (`generate()`, the `main` bridge
section). The deeper issue -- UTF-8 validation in the lexer, correct
`verba` length semantics (characters vs bytes), and `lege` reading
full Unicode codepoints -- requires a dedicated phase.

**Deferred to**: Phase 10b (UTF-8), scheduled right after the stdlib
bootstrap (Phase 10) when string handling is being formalized anyway.

### Future package manager and module path standardization

**Severity**: low (no impact today, design consideration for later)
**Affects**: module system (Phase 9+)

Currently modules are resolved by path relative to the importing file
or via the `HSD_PATH` environment variable. There is no standard
global location for installed packages and no tooling to manage them.

**Planned fix**: a future dev tool (package manager) will manage a
standard global folder (~/.hsd/packages/ or equivalent on Windows).
A per-project `hsd.toml` config file will allow overriding the search
path. The compiler will receive the resolved path list from the tool.
`HSD_PATH` remains the escape hatch for custom setups.

**Deferred to**: post-Phase 22 (after compiler self-hosting), when
tooling becomes the natural next focus.

---

## Resolved issues

### Windows line endings (CRLF)

**Severity**: high (blocked all files created on Windows)
**Affects**: lexer on all platforms when reading Windows files

Files saved on Windows use `\r\n` line endings. The lexer treated
`\r` as an unexpected character and crashed with "unexpected character".

**Fix**: `Lexer::new()` now strips all `\r` characters from the
source before tokenizing. Both `\r\n` and `\n` line endings are
handled identically. One line added in the constructor.

**Resolved in**: Phase 9 (alongside module system work).

### Tab indentation support

**Severity**: medium (blocked all users whose editor defaults to tabs)
**Affects**: lexer on all platforms, most visible on Windows

The lexer rejected tab characters in indentation with the error
"use spaces, not tabs". Most Windows editors (Notepad, VS Code with
default settings) insert tabs when the Tab key is pressed, making
it impossible to write HSD files without manually configuring the
editor.

**Fix**: `handle_indentation()` in `lexer.rs` now counts tabs as
4 spaces. Spaces and tabs can coexist on the same line.

**Resolved in**: Phase 9 (alongside module system work).

---

## How to use this document

When a new rough edge is discovered during development, add it here
with:
- a short title
- severity (low / medium / high)
- what is affected
- a clear description of the symptom
- a known or suspected fix
- what it is deferred to (a phase, or "unknown")

When an issue is fixed, move it to the Resolved section with a note
on what phase or commit addressed it.
