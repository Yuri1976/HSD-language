# HSD — Module System (`affer`)

This document describes the HSD module system as implemented in
Phase 9. It covers the design, the rationale, known limitations,
and future directions.

---

## Overview

HSD uses the keyword `affer` ("to bring", "to carry") to import
another `.hsd` file into the current program. Everything defined in
the imported file — functions, genus types, actor types, constants —
becomes available in the importing file without any prefix or
qualification.

```
affer "lib/geometria"

munus principale() -> nihil
    sit c = crea Cerchio(raggio: 5.0)
    scribe("Area: ", area_cerchio(c))
```

Multiple imports are allowed:

```
affer "lib/geometria"
affer "lib/matematica"
affer "lib/stringhe"
```

---

## How it works

When the HSD compiler loads a file, it scans for `affer` statements
before doing any analysis. Each imported module is loaded recursively
and its items (functions, types, constants) are merged into the main
program before semantic analysis begins.

This means the compiler sees a single flat program — it does not need
to know which symbol came from which file. The semantic analyzer,
interpreter, and C backend all work on this merged view.

Import order matters only for readability: all imported symbols are
available everywhere in the program regardless of where the `affer`
statement appears.

---

## Path resolution

The compiler searches for a module in this order:

1. **Relative to the importing file**: `affer "lib/geometria"` looks
   for `lib/geometria.hsd` in the same directory as the file that
   contains the `affer` statement.

2. **HSD_PATH entries**: for each directory listed in the `HSD_PATH`
   environment variable (colon-separated on Linux/macOS,
   semicolon-separated on Windows), the compiler looks for
   `geometria.hsd` in that directory.

If the module is not found in any of these locations, the compiler
prints a clear error listing every path it searched.

### Setting HSD_PATH

**Linux / macOS:**
```bash
export HSD_PATH=/home/user/.hsd/stdlib:/home/user/mylibs
```

**Windows (PowerShell):**
```powershell
$env:HSD_PATH = "C:\Users\user\.hsd\stdlib;C:\Users\user\mylibs"
```

### Path separators in source

Always use forward slashes in `affer` paths, even on Windows:

```
affer "lib/geometria"    # correct on all platforms
affer "lib\geometria"    # wrong: backslash is an escape character
```

---

## Circular imports

If module A imports module B, and module B imports module A, the
compiler detects the cycle and loads each file only once. The
detection is based on the canonical file path (resolved symlinks and
`..` components), so `affer "lib/../lib/geometria"` and
`affer "lib/geometria"` are correctly recognized as the same file.

---

## Namespace model

HSD uses a **flat namespace**: all imported symbols are merged into
the global scope of the importing program. There are no prefixes or
qualified names.

```
affer "lib/geometria"

munus principale() -> nihil
    sit c = crea Cerchio(raggio: 5.0)   # Cerchio, not geometria.Cerchio
    scribe(area_cerchio(c))             # area_cerchio, not geometria.area_cerchio
```

### Rationale

A flat namespace is simpler to implement, simpler to use, and
consistent with languages like Go, which also uses a single imported
package name without deep qualification. For the current phase of
HSD, where the goal is to enable multi-file programs and a standard
library, flat namespaces are sufficient.

The tradeoff is that name collisions are possible: if two modules
both define a function called `area`, the second one wins silently.
This is acceptable at the current scale. Qualified names
(`geometria.area_cerchio`) are a future extension.

---

## Indentation in module files

Module files follow the same indentation rules as any HSD file.
Tabs and spaces are both accepted; tabs count as 4 spaces. Mixing
tabs and spaces on the same line is allowed.

See `HSD-known-issues.md` for the history of the tab support fix.

---

## Known limitations and future work

### 1. No qualified names (flat namespace only)

**Impact**: name collisions between modules are silently resolved
by last-import-wins. With a small number of well-named modules this
is not a problem, but it does not scale to a large ecosystem.

**Planned fix**: qualified name syntax (`geometria.Cerchio`,
`geometria.area_cerchio`) once the language has a more complete
type system. This requires a namespace-aware symbol table and
changes to field access parsing (`.` is already used for record
fields, so the syntax needs care).

**Deferred to**: post-Phase 15, when the type system is mature
enough to support it cleanly.

### 2. No visibility control (`publicum` / `privatum`)

**Impact**: everything in a module is exported. There is no way to
mark internal helpers as private.

**Planned fix**: `publicum` / `privatum` keywords on function and
type declarations. Items without a visibility modifier default to
`publicum` (open) or `privatum` (closed) — to be decided.

**Deferred to**: Phase 19 (visibility) per the roadmap.

### 3. No package manager

**Impact**: installing and discovering libraries requires manual
file copying or setting `HSD_PATH` by hand. There is no standard
global location for installed packages.

**Planned fix**: a future dev tool will manage a standard folder
(`~/.hsd/packages/` or equivalent). A per-project `hsd.toml` will
allow pinning versions and overriding paths. The compiler will
receive the resolved path list from the tool.

**Deferred to**: post-Phase 22 (compiler self-hosting).

### 4. No separate compilation

**Impact**: importing a large module re-parses and re-analyzes it
every time. For the current scale this is negligible, but it will
become slow as the stdlib grows.

**Planned fix**: a module cache (pre-parsed AST or compiled object
file) keyed by file path and modification time.

**Deferred to**: whenever build times become a real problem
(probably around Phase 13-14 when the stdlib is non-trivial).

---

## Example: splitting a library across files

**`lib/geometria.hsd`** — the library:
```
genus Punto
    x: realis
    y: realis

genus Cerchio
    cx: realis
    cy: realis
    raggio: realis

munus area_cerchio(c: Cerchio) -> realis
    redde 3.14159 * c.raggio * c.raggio

munus distanza_quadrata(a: Punto, b: Punto) -> realis
    sit dx = a.x - b.x
    sit dy = a.y - b.y
    redde dx * dx + dy * dy
```

**`main.hsd`** — the program:
```
affer "lib/geometria"

munus principale() -> nihil
    sit c = crea Cerchio(cx: 0.0, cy: 0.0, raggio: 5.0)
    scribe("Area: ", area_cerchio(c))
    sit p1 = crea Punto(x: 0.0, y: 0.0)
    sit p2 = crea Punto(x: 3.0, y: 4.0)
    scribe("Distanza^2: ", distanza_quadrata(p1, p2))
```

Run with the interpreter:
```
hsd main.hsd
```

Compile to C:
```
hsd build main.hsd
cl main.c runtime\runtime.c /Fe:main.exe /I runtime
main.exe
```
