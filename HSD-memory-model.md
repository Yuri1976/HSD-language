# HSD — Memory Model

This document explains how HSD manages memory, why these choices were
made, and where the limits are. It is meant to serve both as design
documentation for the project and as a reference for future writing
about HSD. The goal is to be precise about the technical content
while staying readable for someone curious about language design
without being a compiler specialist.

---

## In a sentence

HSD uses **Automatic Reference Counting (ARC)** as its memory
management strategy. Every heap-allocated object carries a counter of
how many references point to it; when the counter reaches zero, the
object is freed. The counter is maintained automatically by the
compiler, not by the programmer.

The rest of this document is the long version.

---

## The problem: where does memory come from, and who cleans up?

Every running program needs memory: places to store its values,
strings, lists, structures, intermediate results. Some of this
memory is small and predictable — local variables, function
arguments — and lives on the **stack**, a region that grows and
shrinks automatically as functions are called and return.

But many things in a program are too large or too long-lived to fit
on the stack. A string that grows as a user types. A list whose
length is decided at runtime. A tree of objects that needs to outlive
the function that built it. These live on the **heap**, a region of
memory that the program asks for explicitly and must release when
done.

The question every language has to answer is: **who decides when
heap memory is no longer needed, and frees it?**

Three families of answers have emerged historically.

### Family 1: the programmer decides (manual management)

The classic approach. The programmer writes `malloc` to ask for
memory and `free` to release it. This is what C does.

The advantage: total control, zero runtime overhead. The disadvantage:
mistakes are catastrophic. Free a piece of memory while another part
of the program is still using it (use-after-free), and you have a
crash or a security vulnerability. Free it twice (double-free), and
you corrupt the allocator. Forget to free it (memory leak), and the
program slowly bloats until it dies.

Half a century of C and C++ has shown that even very careful
programmers make these mistakes regularly. Studies of large
codebases find that 60-70% of serious security vulnerabilities come
from memory mismanagement.

### Family 2: a separate system watches and cleans up (garbage collection)

A runtime component — the **garbage collector** — periodically pauses
the program, scans through all reachable memory starting from the
root variables, and frees anything that is no longer reachable. This
is what Java, Go, JavaScript, C#, and most "easy" languages do.

The advantage: the programmer never thinks about freeing memory. The
disadvantages: the runtime has to be sophisticated (sometimes
hundreds of thousands of lines of code), the program experiences
unpredictable pauses while the collector runs, and there is a
constant background cost in memory and CPU even when the program is
working correctly.

For most applications this works fine. For applications with strict
latency requirements (audio processing, game engines, real-time
systems), the pauses are a problem.

### Family 3: each object tracks its own lifetime (reference counting)

A middle path. Every heap-allocated object carries a small counter
that says how many references point to it. When a new reference is
created, the counter goes up by one. When a reference goes out of
scope or is overwritten, the counter goes down by one. When the
counter reaches zero, nothing in the program can reach this object
anymore, and it is freed immediately.

The advantage: deterministic (objects are freed at precise moments,
not "eventually"), no pauses, modest runtime cost. The disadvantage:
the counter updates have a small cost that adds up, and the strategy
on its own cannot handle reference cycles (more on this below).

This is what Swift, Objective-C, and Python use. And it is what HSD
uses.

---

## ARC in detail

Let me walk through what actually happens, mechanically, when an HSD
program manages memory under ARC.

### The allocated block

Whenever HSD creates a heap object (a string, a list, an actor, a
record), it does not just allocate the size of the data. It
allocates a slightly larger block, where the first few bytes hold
the reference count, and the rest holds the actual data.

```
┌─────────────────────────────────────────────────┐
│  refcount  │           actual data              │
│  (8 bytes) │       (size of the value)          │
└─────────────────────────────────────────────────┘
              ↑
              The pointer that HSD code holds
              points to here, not to the start
              of the block
```

The pointer that HSD programs work with points to the data, not to
the refcount. When the runtime needs to update the counter, it
arithmetically steps back 8 bytes from the pointer to find the
counter. This is invisible to the HSD programmer.

### The two fundamental operations

Two operations form the basis of everything: **retain** and
**release**.

A **retain** increments the refcount by 1. It happens when a new
reference to an object is created.

A **release** decrements the refcount by 1. It happens when a
reference goes out of scope or is overwritten. If the decrement
brings the counter to zero, the object is freed.

Here is the simplest possible example of what the compiler generates.

The HSD code:

```romanes
sit nome = "Yuri"
scribe(nome)
```

The C code (conceptually) that ARC generates:

```c
verba* nome = create_string("Yuri");   // refcount starts at 1
arc_retain(nome);                       // refcount becomes 2
scribe(nome);                           // use it
arc_release(nome);                      // refcount becomes 1 (end of scope)
arc_release(nome);                      // refcount becomes 0, freed
```

The exact protocol is more subtle than this (one of those retains is
actually unnecessary in real compilers), but the principle is here:
every interaction with the object is balanced by counter updates.

### A useful image

Here is the image I find most helpful for thinking about ARC. Imagine
a library book. The book itself sits on a shelf, and a small card on
its back records how many people are currently borrowing it. Every
time someone takes it out, the count on the card goes up by one.
Every time someone returns it, the count goes down. When the count
reaches zero, the librarian knows the book is no longer in use and
can decide what to do with it — perhaps reshelve it, perhaps
discard it if no one wanted it for too long.

ARC is exactly this, applied to memory: every object is a "book", the
refcount is the card on its back, and the runtime is the librarian.

### What the compiler has to figure out

The simple description hides where the actual difficulty is: **the
compiler must insert retain and release calls in the right places**,
automatically, for every possible program. Get one wrong and you have
either a leak (if you missed a release) or a crash (if you released
too early). The art is in deciding where to put them.

The rules HSD's compiler will follow are:

1. **When a variable is assigned a heap value, emit a retain.**
2. **When a variable goes out of scope, emit a release.**
3. **When a function receives an argument, no retain or release —
   the argument is "borrowed" from the caller.**
4. **When a function returns a heap value, the caller takes
   ownership — no extra retain by the caller.**
5. **Temporary values within an expression are released at the end
   of the expression.**

Five rules. They look simple, and 80% of code follows them without
incident. The remaining 20% — conditional assignments, loops,
nested structures, exceptions — requires careful handling that we
will look at later.

---

## Why ARC suits HSD

Several reasons converge to make ARC the right fit for HSD
specifically.

### Predictability matches the language's character

HSD aims to be "simple to read, fast to run, with safe concurrency
through actors". ARC fits all three:

- *Simple to read* — the programmer never sees ARC. They write code
  as if memory just worked, like in Python. The compiler inserts the
  bookkeeping.
- *Fast to run* — ARC has small, distributed costs. There are no
  pauses, no surprises, no "the program just stalled for 200ms because
  the garbage collector ran". Every operation pays a tiny amount.
- *Safe concurrency* — ARC works naturally with the actor model.
  Each actor owns its own memory; reference counts only need to be
  atomic when objects cross actor boundaries (which is rare and
  explicit).

### A small runtime is achievable

A tracing garbage collector is a serious piece of engineering. The Go
runtime is around 250,000 lines of code, much of it concerned with
the garbage collector. The Java HotSpot collector is even larger.

ARC is implementable in a few hundred lines. The HSD runtime
(`runtime.c`) today is around 200 lines. Adding ARC adds perhaps
another 100. This stays within the project's didactic, readable
nature — a person can sit down and read the entire HSD runtime in
an afternoon. With a tracing GC this would be impossible.

### Determinism opens design space

When you know exactly when an object is freed, you can attach
behavior to that moment. Swift uses this for "deinit" methods that
run when an object is freed — useful for things like closing files,
releasing locks, removing items from a parent collection. Tracing GC
languages cannot do this reliably, because "when" is not under
control.

HSD does not currently expose this feature, but the design door
stays open.

### Concurrency model alignment

In an actor-based language, the natural way to keep memory safe under
concurrency is "no shared memory". Each actor has its own state and
messages are copied between them.

ARC fits this beautifully: each actor's memory is reference-counted
locally, with no need for atomic operations within an actor (the
actor is single-threaded by design). Atomic counters only become
necessary at the points where memory crosses actor boundaries — and
that is rare and explicit in the language.

A tracing garbage collector for an actor-based language has to be
significantly more complex: either it stops all actors when running
(killing the actor parallelism), or it uses elaborate techniques like
per-actor heaps with cross-heap remembered sets. Erlang's runtime
spent decades optimizing this. HSD avoids the complexity by choosing
ARC from the start.

---

## How other languages handle memory

For context, here is a tour of how the other prominent approaches
compare.

### Comparison table

| Language | Strategy | Runtime cost | Determinism | Cycles | Programmer effort |
|---|---|---|---|---|---|
| **C** | Manual `malloc`/`free` | Zero | Deterministic | N/A | High (and dangerous) |
| **C++** | Manual + smart pointers (`shared_ptr`) | Small (ARC-like for smart pointers) | Deterministic | Leak (use `weak_ptr`) | Moderate |
| **Rust** | Ownership + borrow checker | Zero | Deterministic | Prevented statically | High (steep learning) |
| **Java** | Tracing GC (generational) | Moderate (pauses) | Non-deterministic | Handled | None |
| **Go** | Tracing GC (concurrent) | Low (small pauses) | Non-deterministic | Handled | None |
| **Python** | ARC + cycle detector | Moderate (overhead from dynamic typing) | Deterministic for non-cycles | Detected eventually | None |
| **Swift** | ARC | Small | Deterministic | Leak (use `weak`) | Low (mark cycles manually) |
| **HSD** | ARC + `nativum` opt-out | Small | Deterministic | Leak (future: `tenuus`) | None for normal code |

### Performance: a concrete comparison

Numbers are approximate, drawn from common benchmarks. The test:
"allocate one million short strings and let them be freed". This
stresses memory management without doing other work.

| Language | Approximate time |
|---|---|
| C (with `malloc`/`free`) | 50 ms (baseline) |
| Rust (ownership) | 50 ms |
| Swift (ARC) | 80 ms (1.6× baseline) |
| HSD (projected, ARC) | 80 ms (similar to Swift) |
| Go (tracing GC) | 100 ms (2× baseline) |
| Java (tracing GC) | 120 ms (2.4× baseline) |
| Python (ARC + interpreter overhead) | 800 ms (16× baseline) |

A few things to notice:

- Rust achieves C-speed because ownership is compile-time only —
  the binary has no runtime ARC mechanism. The cost was paid by the
  programmer in writing.
- HSD's projected performance sits with Swift, around 1.5-2× the cost
  of C/Rust for pure allocation-heavy code. For most real programs
  (where allocation is a small fraction of total work), the
  difference is invisible.
- Python is much slower, but not because of the refcounting itself.
  The cost is the interpreter on top. Python's refcounter is
  roughly as fast as Swift's.

### Memory safety: a feature-by-feature comparison

| Feature | C | C++ | Rust | Java/Go | Swift | HSD |
|---|---|---|---|---|---|---|
| Use-after-free prevention | ❌ | ⚠️ | ✅ | ✅ | ✅ | ✅ |
| Double-free prevention | ❌ | ⚠️ | ✅ | ✅ | ✅ | ✅ |
| Memory leak prevention | ❌ | ⚠️ | ✅* | ✅ | ⚠️ cycles | ⚠️ cycles |
| Buffer overflow prevention | ❌ | ❌ | ✅ | ✅ | ✅ | ✅ |
| Data race prevention | ❌ | ❌ | ✅ (static) | ❌ | ❌ | ✅ (via actors) |
| Cost paid for safety | None | Some | Compile-time | Runtime (GC) | Small runtime | Small runtime |

*Rust can leak through `Rc<RefCell<T>>` cycles, but it requires
explicit construction.

The asterisk on Rust is important. Rust often gets credit for
"complete memory safety", but the cycle case shows it is not quite
complete — and there are situations where the borrow checker refuses
code that is actually correct, forcing the programmer to use
`unsafe` blocks. The trade-off is different from HSD's, not strictly
better.

---

## The honest limits of ARC

Three real limitations of ARC, presented in order of importance.

### Limit 1: reference cycles

This is the famous one. If object A holds a reference to object B,
and B holds a reference back to A, both refcounts will always be at
least 1 — even when nothing else in the program can reach either of
them. They remain in memory forever. Leak.

A concrete example: a doubly linked list. Each node points to the
next node (which the node "owns") and to the previous node (which
points back). The pair of pointers between adjacent nodes forms a
cycle.

```
Node1 ──→ Node2 ──→ Node3
  ↑↓        ↑↓        ↑↓
  └─────────┴─────────┘
```

In pure ARC, this list never gets freed even when the program no
longer holds a reference to its head.

This is not a theoretical problem — many natural data structures
have this shape: trees with parent pointers, observer patterns,
graphs, caches with two-way lookup.

**HSD's plan for cycles** is documented separately (Phase 15b in the
roadmap): introduce **weak references** as an explicit opt-in.
Syntactically, the proposed keyword is `tenuus` (Latin for
"weak" / "slender"). A `tenuus[T]` is a reference that does not
increment the refcount. When the strong refcount of the target
reaches zero, all `tenuus` references to it automatically become
`nihil`. The programmer marks the back-pointers in cyclic structures
as `tenuus`, and the cycle is broken.

```romanes
genus Nodo
    valore: numerus
    figlio: Nodo                # strong reference
    genitore: tenuus[Nodo]      # weak reference, no refcount increase
```

This is the same model Swift uses (`weak var`), the same C++ uses
(`weak_ptr`), the same Rust uses (`Weak<T>`). It works.

Until weak references arrive, the limitation is real but small: most
programs do not naturally create cycles. Web back-ends, CLI tools,
data pipelines, compilers — none of these typically have cyclic data.
Cycles are a problem for specific domains (UI frameworks, certain
data structures) that will benefit from `tenuus` once it lands.

### Limit 2: small but constant overhead

Every retain and release costs a few instructions: load the refcount,
modify it, store it back. On modern CPUs this is around 2-5
nanoseconds. Individually negligible, but in code that creates and
destroys many small objects in a hot loop, the overhead can add up.

The rough magnitudes:

- For a typical program (web server, data tool, compiler), ARC
  overhead is usually under 5% of total runtime. Invisible.
- For an unusually allocation-heavy program (a parser allocating
  millions of AST nodes), ARC might be 10-20% of runtime. Noticeable
  but not catastrophic.
- For a hot graphics or simulation loop allocating millions of
  vectors per second, ARC can be 30-50% of runtime. This is where it
  hurts.

HSD's answer for the last case is `nativum`: a block where ARC is
disabled, where the programmer takes responsibility for memory
manually, in exchange for raw C-equivalent speed. This is covered in
the next section.

### Limit 3: cache effects of distributed refcount updates

A subtler limitation, less often discussed. When you do
`arc_retain(obj)`, the CPU has to load the cache line containing
`obj`'s refcount. If that cache line is not in your CPU's local
cache, you pay a cache miss (50-200 nanoseconds). In multi-threaded
code where multiple threads touch the same objects, the cache line
bounces between cores — even more expensive.

Tracing GC has a different cache profile: it can do all its work in
one pass, touching memory in a more cache-friendly order. For very
large heaps, this matters.

In practice, for the kinds of programs HSD is targeting (medium-sized
heaps, mostly single-actor work), this is rarely the dominant cost.
But it is worth knowing about.

---

## HSD's strategy for the limits

To summarize how HSD handles each limitation:

**For cycles → `tenuus` (weak references), Phase 15b.**
Explicit opt-in. The programmer marks back-pointers as weak; the
runtime breaks the cycle on deallocation. Coherent with HSD's
preference for opt-in complexity, deterministic, zero cost for code
that does not use them.

**For overhead in hot paths → `nativum` blocks.**
Already in the language design. A `nativum` block disables ARC, lets
the programmer manage memory by hand (or use stack allocation), and
gives C-equivalent performance. Useful for tight numeric loops, SIMD
code, graphics primitives. Opt-in, explicit, scoped: the rest of the
program remains ARC-managed.

**For cache effects → architectural decisions.**
The actor model encourages keeping working sets small (each actor's
state is bounded, messages are copied, no shared mutable state).
This is naturally cache-friendly. Not a "fix" to ARC, but a design
that makes the limitation matter less.

---

## The decisions for HSD's implementation

This section pins down the specific choices made for Phase 6 of the
HSD roadmap, when ARC is being implemented.

### What is heap-tracked

Only types that live on the heap need ARC. Primitives (numbers,
booleans, `nihil`) are passed by value on the stack and need no
counter.

| Type | Heap? | ARC-tracked? |
|---|---|---|
| `numerus` | No (stack value) | No |
| `realis` | No (stack value) | No |
| `veritas` | No (stack value) | No |
| `nihil` | N/A | No |
| `verba` (string) | Yes | **Yes** |
| `series[T]` (list) | Yes | **Yes** |
| `genus` (record) | Yes (future) | **Yes** |
| `actor` | Yes | **Yes** |

### Header layout: inline, 8 bytes

Every heap-tracked allocation reserves the first 8 bytes for the
refcount. The HSD-visible pointer points to the data, 8 bytes after
the start of the allocation. The runtime reads and updates the
refcount by stepping back from the pointer.

This is the same layout Swift and CPython use. Simple, fast, no
auxiliary data structures.

### Atomicity: non-atomic for now

The refcount is stored as a regular 64-bit integer, with regular
increment and decrement operations. This is fast (a single
instruction) but not safe across threads.

This works for Phase 6 because HSD's actors are still synchronous
(introduced in Phase 5c). There is no real concurrency yet, so no
two cores will ever race on the same refcount.

In Phase 21, when real actor concurrency arrives, the refcount will
become atomic. This is a planned migration, marked with `TODO`
comments in the source.

### Ownership convention: borrowing on parameters

When a function takes an argument of a heap-tracked type, **no
retain or release is emitted**. The argument is "borrowed" from the
caller for the duration of the call. The caller is responsible for
keeping it alive (which it does naturally — the value is in a
variable in the caller's scope).

This is the same convention Rust uses (with `&T` borrowing) and
Swift uses (with `+0` argument conventions). It is significantly more
efficient than the alternative of "every parameter passing causes a
retain/release pair", and it does not change what the programmer
writes.

### Return values: ownership transfer

When a function returns a heap-tracked value, the value is returned
with refcount already incremented. The caller takes ownership without
needing an extra retain.

```romanes
munus crea_saluto() -> verba
    sit s = formatta("Hello, world")
    redde s
```

The function builds `s` (refcount 1 from creation), then returns it.
The caller receives it with refcount 1 already correct. No extra
operations.

### Cleanup: inline at every `redde`

When a function has multiple `redde` statements, each one emits the
release calls for all local variables in scope at that point. This
produces verbose C output but is straightforward to debug.

The alternative — rewriting every function to have a single exit
point with a unified cleanup block — was rejected because it makes
the generated C harder to follow.

### Cycles: documented, deferred

For Phase 6, cycles are not handled. Programs that create them will
leak. This is documented in the project overview and in the source
code where ARC is implemented (with a comment pointing to Phase 15b).
The plan is `tenuus` (weak references) in Phase 15b.

---

## Examples: the same operation across languages

To make the differences concrete, here is one operation — building a
string from two parts and printing it — written in five languages.

### C (manual memory)

```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

char* concat(const char* a, const char* b) {
    size_t len = strlen(a) + strlen(b);
    char* result = malloc(len + 1);
    strcpy(result, a);
    strcat(result, b);
    return result;
}

int main(void) {
    char* greeting = concat("Hello, ", "world");
    printf("%s\n", greeting);
    free(greeting);    // forget this line → leak
    return 0;
}
```

The programmer must remember to `free`. Forget it: leak. Free too
early: crash. The compiler does not help.

### C++ (smart pointers)

```cpp
#include <iostream>
#include <memory>
#include <string>

int main() {
    auto greeting = std::make_shared<std::string>("Hello, " "world");
    std::cout << *greeting << std::endl;
    // automatic release when shared_ptr goes out of scope
    return 0;
}
```

The smart pointer handles the release automatically. Cycles between
`shared_ptr` still leak, requiring `weak_ptr` to break.

### Rust (ownership)

```rust
fn main() {
    let greeting = format!("{}{}", "Hello, ", "world");
    println!("{}", greeting);
    // greeting is automatically dropped at end of scope
}
```

Ownership is implicit in `let`. No counter, no runtime check. The
compiler proves at build time that the value is dropped exactly once.
If the programmer tries to use `greeting` after some operation that
would invalidate it, the compiler refuses.

### Python (reference counting under the hood)

```python
def main():
    greeting = "Hello, " + "world"
    print(greeting)
    # refcount drops to 0 when main returns, object is freed
main()
```

The refcounter is there but invisible. The programmer never sees it.
Python's cycle detector handles the rare cyclic cases.

### HSD

```romanes
munus principale() -> nihil
    sit greeting = formatta("Hello, ", "world")
    scribe(greeting)
    # refcount drops to 0 when principale returns, object is freed
```

Syntactically very close to Python. Semantically very close to Swift.
The programmer writes natural code; the compiler inserts the retain
and release calls.

What the HSD compiler generates (conceptually):

```c
void principale(void) {
    verba* greeting = formatta("Hello, ", "world");  // refcount=1
    // (no retain — already created with count 1)
    scribe(greeting);
    arc_release(greeting);  // refcount=0, freed
}
```

---

## Glossary

A few terms used throughout this document and in HSD source code.

**ARC** — Automatic Reference Counting. A memory management strategy
where the compiler inserts code to track how many references point
to each heap object, freeing it when the count reaches zero.

**Allocation** — Asking the operating system (or an allocator like
`malloc`) for a chunk of memory. The opposite is *deallocation* or
*freeing*.

**Heap** — A region of memory used for objects whose lifetime
exceeds the function that created them, and whose size may not be
known at compile time. Allocations are explicit; deallocations
depend on the memory strategy.

**Stack** — A region of memory used for local variables and
function call frames. Memory is automatically reclaimed when a
function returns. Very fast, no management needed, but limited in
size and lifetime.

**Reference** — A pointer to a heap-allocated object. Multiple
references can point to the same object; ARC tracks how many.

**Refcount** — Short for "reference count". The integer attached to
each heap-allocated object recording how many references currently
point to it.

**Retain** — Increment the refcount by 1.

**Release** — Decrement the refcount by 1. If the result is zero,
free the object.

**Strong reference** — A reference that participates in refcounting.
This is the default.

**Weak reference** (`tenuus` in HSD, planned) — A reference that
does not participate in refcounting. Used to break cycles. When the
referenced object is freed, the weak reference becomes `nihil`
automatically.

**Cycle** — A loop in the graph of references where A points to B,
B points back to A (directly or through other objects). Pure ARC
cannot detect or break cycles; weak references are the standard
solution.

**Garbage Collection (GC)** — Strictly, any automatic memory
management. Colloquially, *tracing garbage collection*: a separate
runtime component that periodically scans for unreachable memory and
frees it. Distinct from ARC.

**Tracing GC** — The form of garbage collection used by Java, Go,
JavaScript, C#. The collector traces references from roots, marks
reachable objects, and frees everything else. Causes occasional
pauses.

**Use-after-free** — Accessing an object after it has been freed.
Causes undefined behavior. ARC and GC both prevent this; manual
memory management does not.

**Double-free** — Freeing the same object twice. Corrupts the
allocator. ARC and GC prevent this; manual memory management does
not.

**Memory leak** — An allocation that is never freed even though the
program no longer needs it. ARC handles most leaks automatically;
cycles are the exception.

**Data race** — Two threads accessing the same memory concurrently,
with at least one of them writing. Causes undefined behavior. ARC
and GC do not prevent data races; HSD's actor model does, by
preventing shared mutable state between actors.

**`nativum` block** — In HSD, a syntactically distinct block where
ARC is disabled and the programmer takes manual control of memory.
Used for performance-critical code (graphics inner loops, SIMD,
manual allocators). The rest of the program remains ARC-managed.

---

## References and further reading

- Apple's documentation on Swift's ARC implementation:
  `https://docs.swift.org/swift-book/documentation/the-swift-programming-language/automaticreferencecounting/`
- Wilson, P. R. (1992). *Uniprocessor Garbage Collection
  Techniques.* The classic survey of memory management strategies.
- Bacon, D. F., Cheng, P., Rajan, V. T. (2003). *A Unified Theory
  of Garbage Collection.* Shows that tracing GC and reference
  counting are mathematical duals.
- Boehm, H. (2004). *The Space Cost of Lazy Reference Counting.*
  On the subtleties of when objects actually get freed under ARC.
- The Rust Book, chapter on "Smart Pointers", which covers `Rc`,
  `RefCell`, `Weak` — Rust's library-level reference counting,
  used in cases where the compile-time ownership system is too
  restrictive.
