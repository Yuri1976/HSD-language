# HSD — ARC Technical Debt

A running inventory of known limitations and unresolved cases in
HSD's ARC implementation. Items here are deliberately deferred or
discovered after the initial implementation; this document tracks
them so they are not forgotten and so contributors can see what is
known.

For the design rationale of ARC itself, see `HSD-memory-model.md`.
For the implementation phases, see `HSD-roadmap.md`.

---

## Active debt

Items that should be addressed in the foreseeable future.

### 1. Temporary heap values are not released

**Severity:** Medium.

**Problem.** When a heap-returning expression appears in an outer
expression and its result is not stored in a variable, the temporary
value leaks.

Examples:

```romanes
scribe(crea_lista())              # the returned list is never released

per x in numera(numera(1, 5)[2], 10)   # the inner numera leaks

si f() et g()                     # if f or g return heap values, they leak
    ...
```

**Why it happens.** The codegen generates code for expressions but
does not introduce temporary variables to hold intermediate heap
values, so it never has a name to release.

**When to fix.** Before user functions start returning heap values
routinely. As of Phase 6, the only callable functions that return
heap values are `numera` (returns `hsd_list_num`) and `lege`
(returns `const char*`), both used in patterns that already store
their result.

**How to fix.** Introduce synthetic temporary variables in the
codegen for any sub-expression of heap-tracked type. Release them
at the end of the containing statement. Requires `gen_expr` to be
context-aware (it must know what statement-level scope to release
into).

---

### 2. User functions returning heap values: limited testing

**Severity:** Low (probably works, untested).

**Problem.** Functions like:

```romanes
munus crea_saluto() -> verba
    redde "ciao"
```

should work correctly: the literal is wrapped by `hsd_arc_copy_str`,
returned with refcount 1, and the caller's scope tracks it. But this
flow has not been validated with a test program.

**When to fix.** Before relying on user functions for heap-typed
returns. A test program would help confirm the behaviour and catch
edge cases.

**How to fix.** Write a test that exercises function return of
strings and lists, possibly chained through multiple function calls,
and verify memory stability under a stress loop.

---

### 3. `if` with all branches returning emits dead code

**Severity:** Cosmetic.

**Problem.** Consider:

```romanes
munus f() -> numerus
    si cond
        redde 1
    aliter
        redde 2
```

`block_ends_with_return` only checks if the last statement of the
function body is a `Return`. It does not detect that the last
statement is an `If` whose every branch terminates with `Return`.
As a result, the codegen emits scope cleanup after the `if/else`
block, which is unreachable.

**Impact.** The C compiler will emit an unreachable-code warning.
The program still runs correctly (the cleanup is dead code, never
executed).

**When to fix.** Whenever the warning becomes irritating.

**How to fix.** Make `block_ends_with_return` recursive: an `If`
where every branch (`then_block`, all `elif`, and `else_block`)
ends with a return counts as "ends with return". Same logic could
extend to `Match` once pattern matching exists.

---

## Architectural debt — known but planned

These are not bugs; they are deliberate deferrals tracked in the
main roadmap.

### Heap-typed fields in actors are not released

When an actor is freed, its state fields that are themselves
heap-tracked (`verba`, `series`, future `genus` values) are not
released. The actor's memory itself is freed, but the data those
fields pointed to leaks.

**Plan:** Phase 8 (`genus` end-to-end). When struct destructors are
implemented for `genus`, the same machinery applies to actor state.

### Refcount is non-atomic

Reference counts are stored as plain `long`, with non-atomic
increment and decrement operations. Safe today because HSD's actors
are synchronous (Phase 5c). Unsafe under real concurrency.

**Plan:** Phase 21 (real actor concurrency). Refcounts become
`_Atomic long` or use platform-specific atomic intrinsics.

### Reference cycles leak

ARC cannot detect or break cycles. A doubly linked list, a tree
with parent pointers, or any cyclic graph will leak.

**Plan:** Phase 15b (weak references). The keyword `tenuus[T]`
will mark non-counting back-references, allowing cycles to be
broken explicitly without runtime cycle detection.

---

## Non-bugs (clarifications)

Items that may look like debt but are working as designed.

### String literals passed to printf are not wrapped

When a string literal appears as a `scribe` argument:

```romanes
scribe("hello", n)
```

the generated C is:

```c
printf("%s%ld\n", "hello", n);
```

The literal is *not* wrapped in `hsd_arc_copy_str`. This is correct:
`printf` reads the string and discards the pointer, so there is
nothing to release. Wrapping would only allocate and leak.

### Parameters are not released at function exit

Function parameters are deliberately borrowed from the caller. The
caller owns the value; the callee uses it without retaining or
releasing.

```romanes
munus stampa(s: verba) -> nihil
    scribe(s)
```

generates C that does not call `hsd_arc_release(s)` on exit. The
caller's scope is responsible for releasing the value. This is the
standard "borrowing" convention from Swift and Rust.

### Inside `nativum` blocks, ARC is suspended

Code inside `nativum { ... }` is generated without any retain/release
calls. The programmer takes manual responsibility for memory inside
these blocks. This is the intended behaviour of the opt-out escape
hatch.

---

## How to use this document

When implementing a new HSD feature that interacts with ARC,
check whether it touches any of the items above. When fixing one
of these items, move it out of "Active debt" into a brief mention
in `HSD-memory-model.md` under "implementation status", with the
phase number where it was resolved.

This document should shrink over time, not grow.
