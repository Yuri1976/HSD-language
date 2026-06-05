# HSD — A Comparison with Python, C, and Rust

A four-way comparison meant to place HSD in a landscape of languages
most readers already know. Code first, then analysis: the factorial
function written in all four languages, then a systematic look at the
paradigms, types, vocabularies, and design choices that set them apart.

---

## The four languages, in brief

**Python** (1991, Guido van Rossum). General-purpose, high-level,
**bytecode-interpreted** on a virtual machine (CPython). Dynamic
typing: types live on values, not on variables. Philosophy:
readability, simplicity, "one thing, one way". Dominant in scripting,
data science, automation, web back-ends, and teaching.
*In a sentence:* "Write it the natural way; the VM will handle it."

**C** (1972, Dennis Ritchie). Systems language, **compiled to native**,
static, low-level. Manual memory management. Philosophy: closeness to
the metal, minimalism, total trust in the programmer (even when
wrong). The foundation of Unix, Linux, embedded systems, and most of
the world's digital infrastructure.
*In a sentence:* "No illusions. The machine is right here; you tell
it what to do."

**Rust** (1.0 in 2015, originally Mozilla). Systems language,
**compiled to native**, statically and very strongly typed. Memory
safety guaranteed *without* a garbage collector, through the
**ownership** and **borrow checker** systems. Philosophy: C's
performance without its dangers, concurrency without data races.
*In a sentence:* "C-speed, but without shooting yourself in the foot —
even if it costs you up front."

**HSD** (this project — didactic state). Statically typed with **type
inference**, Latin keywords, ARC and actors as the memory and
concurrency models, **compiles to native code by translating to C**.
Philosophy: a small keyword core, clear semantics, readability without
giving up native speed.
*In a sentence:* "Simple syntax, strict checks, fast executable —
with actors built into the grammar."

---

## The same program, four ways

The textbook problem: compute the factorial of 5 and print it.

### Python

```python
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

print("Factorial of 5:", factorial(5))
```

### C

```c
#include <stdio.h>

long factorial(long n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

int main(void) {
    printf("Factorial of 5: %ld\n", factorial(5));
    return 0;
}
```

### Rust

```rust
fn factorial(n: u64) -> u64 {
    if n <= 1 {
        return 1;
    }
    n * factorial(n - 1)
}

fn main() {
    println!("Factorial of 5: {}", factorial(5));
}
```

### HSD

```romanes
munus factorial(n: numerus) -> numerus
    si n <= 1
        redde 1
    aliter
        redde n * factorial(n - 1)

munus principale() -> nihil
    sit result = factorial(5)
    scribe("Factorial of 5: ", result)
```

---

## What the four programs reveal

### Line count

| Language | Non-empty lines |
|---|---|
| Python | 5 |
| HSD    | 8 |
| Rust   | 9 |
| C      | 11 |

Python is the most compact, C the most verbose. HSD sits between Rust
and Python — closer to Rust on line count, but the lines themselves
read closer to Python's. Compactness alone is not the right measure:
what matters is whether the lines feel dense or light.

### Preamble

Python, Rust, and HSD need nothing at the top of the file. C **must**
declare `#include <stdio.h>`, or it does not know what `printf` is.
This is the cost of explicitness that the other three avoid: they
come with an implicit standard library. `print`, `println!`, and
`scribe` are known without being imported.

### Entry point

| Language | Entry point |
|---|---|
| Python | none — top-level code runs at startup |
| C      | `int main(void) { ... return 0; }` |
| Rust   | `fn main() { ... }` |
| HSD    | `munus principale() -> nihil` |

Python is the only one without a mandatory "main" function — code
outside of any function runs when the program starts. The other three
require a designated entry function. HSD follows the C/Rust convention,
consistent with its compiled nature.

### Types

Python writes types nowhere: it discovers them at runtime. C writes
them **everywhere**: every parameter, every return, every local.
Rust and HSD sit in the middle — types required on signatures,
inferred on locals:

- Rust: `fn factorial(n: u64) -> u64` — but `let x = factorial(5)`
  (no `u64` on the local).
- HSD: `munus factorial(n: numerus) -> numerus` — and
  `sit result = factorial(5)`.

It is the combination HSD borrows from Rust: explicit contracts
between functions, lightness inside them.

### Braces vs indentation

Python and HSD use **indentation** for blocks — the structure of the
code is its visual shape. C and Rust use **curly braces** `{ ... }` —
the structure is explicit in punctuation.

```python
if n <= 1:
    return 1
```

```c
if (n <= 1) {
    return 1;
}
```

Two philosophies. Indentation forces clean alignment and produces
cleaner code by default; braces are stricter and less ambiguous for
the compiler (and for any reader confronted with poorly indented
code).

### Semicolons

Python and HSD have none. C has them **everywhere**: every statement
ends in `;`. Rust does too — but with an interesting subtlety.

Look at the last line of `factorial` in Rust:

```rust
n * factorial(n - 1)
```

**No semicolon.** Without `;`, that expression *is* the function's
return value. It is a kind of "implicit return": the last expression
of a block without `;` becomes the result of the block. Rust mixes
imperative style with an idea from functional languages. HSD does not
have this shortcut: returns are always explicit, with `redde`.

### Printing to the screen

Four philosophies:

| Language | Syntax | Approach |
|---|---|---|
| Python | `print("Factorial of 5:", factorial(5))` | variadic, comma-separated, auto space |
| C | `printf("Factorial of 5: %ld\n", factorial(5))` | explicit format string with type placeholders |
| Rust | `println!("Factorial of 5: {}", factorial(5))` | macro, `{}` as a generic placeholder |
| HSD | `scribe("Factorial of 5: ", result)` | variadic, concatenation, auto newline |

C is the strictest (and the easiest to break: get the placeholder
wrong and you print garbage). Python and HSD are the most "human":
throw whatever you want in, the language figures out how to render
it. Rust sits in the middle with its macro — safer than C (it checks
types at compile time) but more structured.

### Recursion

The four versions are **structurally identical**: base case
`n <= 1 → 1`, recursive case `n * factorial(n - 1)`. Recursion is a
universal idea, and all four express it the same way. What changes is
not *what* the program does — it is *how you say it*.

---

## Adding input — the languages diverge more

The pure factorial program hides a lot. Adding interactive input —
asking the user for the number, parsing it, computing, printing — is
where each language's philosophy becomes harder to hide.

### Python

```python
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

text = input("Enter a number: ")
n = int(text)
print(f"The factorial of {n} is {factorial(n)}")
```

### C

```c
#include <stdio.h>
#include <stdlib.h>

long factorial(long n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

int main(void) {
    char buffer[64];
    printf("Enter a number: ");
    fflush(stdout);
    fgets(buffer, sizeof(buffer), stdin);
    long n = atol(buffer);
    printf("The factorial of %ld is %ld\n", n, factorial(n));
    return 0;
}
```

### Rust

```rust
use std::io::{self, Write};

fn factorial(n: u64) -> u64 {
    if n <= 1 {
        return 1;
    }
    n * factorial(n - 1)
}

fn main() {
    print!("Enter a number: ");
    io::stdout().flush().unwrap();
    let mut text = String::new();
    io::stdin().read_line(&mut text).unwrap();
    let n: u64 = text.trim().parse().unwrap();
    println!("The factorial of {} is {}", n, factorial(n));
}
```

### HSD

```romanes
munus factorial(n: numerus) -> numerus
    si n <= 1
        redde 1
    aliter
        redde n * factorial(n - 1)

munus principale() -> nihil
    sit text = lege("Enter a number: ")
    sit n = numerus_ex(text)
    scribe("The factorial of ", n, " is ", factorial(n))
```

### The cost of I/O

Adding input makes every program longer, but **not by the same
amount**.

| Language | Non-empty lines | I/O cost |
|---|---|---|
| Python | 7  | none (`input` is already available) |
| HSD    | 9  | none (`lege` is in the standard library) |
| Rust   | 15 | requires `use std::io::{self, Write};` |
| C      | 17 | requires `<stdio.h>` and `<stdlib.h>` |

Python and HSD pay *nothing* for keyboard input: `input` and `lege`
are visible without imports. Rust must open `std::io` explicitly. C
needs two headers (`stdio.h` for `printf`/`fgets`, `stdlib.h` for
`atol`). The gap widens dramatically: Python and HSD grow by about
two lines, Rust and C by six or more.

### The hidden flush

A subtlety that bites in three of the four languages:

```c
printf("Enter a number: ");
fflush(stdout);   // <-- without this, the prompt may not appear
```

```rust
print!("Enter a number: ");
io::stdout().flush().unwrap();   // <-- same idea
```

When you print a prompt **without a newline**, the system holds it in
a buffer and does not show it immediately. If you ask for input next,
the program looks frozen — it is waiting, but you do not see the
prompt. The fix is to explicitly *flush the buffer*. Python does this
inside `input()`. **HSD does it inside `lege`** — hidden on purpose,
because it is the kind of boring detail a language should spare from
the programmer.

### What happens on bad input

If the user types `hello` instead of a number:

| Language | Behavior |
|---|---|
| Python | `int("hello")` raises a `ValueError`: program stops with a clear message |
| Rust | `.parse().unwrap()` panics: program stops with a stack trace |
| HSD | `numerus_ex("hello")` raises a runtime error: program stops with a message |
| C | `atol("hello")` silently returns **0**: program computes "the factorial of 0" and says nothing |

This is one of the places C shows its character: safety is the
programmer's responsibility, not the language's. The other three
protect the programmer — in different ways (exceptions, panic,
runtime error) — but they all say something when things go wrong.

### Input length

C has one more hidden debt: `char buffer[64]` reserves 64 bytes and
no more. If the user types a 100-digit number, the program writes
*beyond* the buffer — undefined behavior, in practice a crash or, far
worse, a security hole. Python, Rust, and HSD handle inputs of
arbitrary length without anything special: the system grows the
string as needed.

### HSD's lineage, shown in three lines

Put the four programs side by side and HSD's choices read out plain.
Compare the three input lines:

```python
# Python
text = input("Enter a number: ")
n = int(text)
print(f"The factorial of {n} is {factorial(n)}")
```

```romanes
# HSD
sit text = lege("Enter a number: ")
sit n = numerus_ex(text)
scribe("The factorial of ", n, " is ", factorial(n))
```

**Structurally identical.** `input` ↔ `lege`, `int(...)` ↔
`numerus_ex(...)`, `print` ↔ `scribe`. The kinship is explicit, by
design — Python is the readability model HSD set out to match.

But under the hood HSD is not Python: the types are static, inference
deduces them silently, and the final executable is native, just like
C's. The synthesis the project has always aimed for: **read like
Python, run like C, checked like Rust**.

---

## A note on actors

The three languages compared here — Python, C, Rust — are the ones
most readers know, and they are good company for showing most of HSD's
choices. But none of them is the right reference for HSD's actor
model. For that, the language to read is **Erlang** (and its modern
descendant Elixir). Erlang has run telecommunication systems for
forty years on a principle HSD borrows directly: actors do not share
memory, and that is the safest way to avoid memory races — make
shared memory impossible at the language level. The actor-related
syntax in HSD (`actor`, `accipe`, `mitte`, `nuntius`) is Erlang's
idea in Latin dress. Python, C, and Rust offer actor models through
libraries; in HSD, as in Erlang, actors live in the grammar.

---

## Programming paradigms

| Paradigm | Python | C | Rust | HSD |
|---|---|---|---|---|
| **Imperative / procedural** | ✅ | ✅ | ✅ | ✅ |
| **OOP** (classes, methods) | ✅ full | ❌ | ⚠️ no inheritance | ❌ (not planned) |
| **Functional** (first-class functions, closures) | ⚠️ partial | ❌ | ✅ closures, iterators | ❌ (today) |
| **Generic** (type parameters) | ✅ duck typing | ⚠️ macros only | ✅ generics + traits | 🔮 planned (via "inflection") |
| **Actor-based concurrency** | ⚠️ via libraries | ⚠️ via libraries | ⚠️ via libraries | ✅ **native** |

Actors-as-a-primitive is HSD's most distinctive paradigm choice: the
other three offer the model through libraries, HSD puts it in the
*grammar* of the language.

---

## Primitive and built-in types

| Concept | Python | C | Rust | HSD |
|---|---|---|---|---|
| Integer | `int` (arbitrary precision) | `int`, `long`, `long long` | `i8`–`i128`, `u8`–`u128` | `numerus` |
| Floating point | `float` | `float`, `double` | `f32`, `f64` | `realis` |
| Boolean | `bool` | `_Bool` / `bool` (C99) | `bool` | `veritas` |
| Single character | (none) | `char` | `char` (4-byte Unicode) | (none) |
| String | `str` | `char*` | `String` / `&str` | `verba` |
| No value | `None` | `void` | `()` (unit) | `nihil` |
| List | `list` | fixed-size array | `Vec<T>` | `series[T]` |
| Tuple | `tuple` | `struct` (manual) | tuple `(T, U, ...)` | ⏳ planned |
| Map / dictionary | `dict` | (manual) | `HashMap<K, V>` | ⏳ planned |
| Set | `set` | (no) | `HashSet<T>` | (not planned) |
| Record / structure | `class` / `dataclass` | `struct` | `struct` | `genus` |
| Variant / union | `Union`, `Enum` | `enum`, `union` | `enum` (with data!) | (not planned) |
| First-class function | ✅ | function pointers | closures | ❌ |
| Shared reference | implicit | pointer `*` | `&` / `&mut` | ARC (implicit) |

One note: Python's `int` has arbitrary precision (it can represent
numbers as large as you want), while HSD's `numerus`, like C's
`long`, has a fixed size (64 bits). A trade-off: arbitrary precision
has a cost; fixed size is fast.

---

## Vocabulary — the keywords

The length of the keyword list says a lot about a language's
philosophy.

### Python (35 keywords)
```
False  None  True  and  as  assert  async  await  break  class
continue  def  del  elif  else  except  finally  for  from  global
if  import  in  is  lambda  nonlocal  not  or  pass  raise
return  try  while  with  yield  match  case
```

### C (~44 keywords, C17 standard)
```
auto  break  case  char  const  continue  default  do  double  else
enum  extern  float  for  goto  if  inline  int  long  register
restrict  return  short  signed  sizeof  static  struct  switch
typedef  union  unsigned  void  volatile  while
_Alignas  _Alignof  _Atomic  _Bool  _Complex  _Generic
_Imaginary  _Noreturn  _Static_assert  _Thread_local
```

### Rust (~50, both active and reserved)
```
as  async  await  break  const  continue  crate  dyn  else  enum
extern  false  fn  for  if  impl  in  let  loop  match
mod  move  mut  pub  ref  return  Self  self  static  struct
super  trait  true  type  unsafe  use  where  while
+ reserved for the future: abstract, become, box, do, final,
  macro, override, priv, try, typeof, unsized, virtual, yield
```

### HSD (27 keywords)
```
sit  fixum  munus  redde  si  aliter  dum  per  in
frange  perge  verum  falsum  et  vel  non  nihil
genus  actor  nuntius  accipe  mitte  ad  crea  ipse
affer  nativum
```

The type names (`numerus`, `realis`, `veritas`, `verba`, `series`)
are **pre-declared identifiers**, not actual keywords: the lexer
treats them as ordinary identifiers, the semantic analyzer
recognizes them. This keeps the grammatical core lean — and lets the
standard library grow (`scribe`, `lege`, `numera`, and whatever comes
next) without touching the lexer or parser. **HSD has the smallest
vocabulary of the four**, on purpose.

---

## The full comparison

| Feature | Python | C | Rust | HSD |
|---|---|---|---|---|
| **Typing** | dynamic | static | static | static |
| **Type inference** | (n/a) | none | local, strong | local |
| **Type safety** | runtime | weak (free casts) | very strong | strong (in progress) |
| **Execution** | interpreted (bytecode + VM) | compiled to native | compiled to native | translated to C → compiled to native |
| **Memory safety** | yes (GC) | none | yes (static, ownership) | yes (ARC — designed, not yet implemented) |
| **Memory management** | garbage collector | manual (`malloc`/`free`) | ownership + borrow checker | ARC + `nativum` opt-out |
| **Concurrency** | threading + GIL, asyncio | pthreads, manual | thread-safe, async, no data races | actors (native) |
| **Typical speed** | low (~50× slower than C) | maximum | on par with C | goal: on par with C |
| **Compile time** | none (fast startup) | fast | slow (heavy checks) | medium (transpile + GCC) |
| **Learning curve** | gentle | medium | steep | gentle (the goal) |
| **Syntax** | indentation, readable | braces, terse | braces, sigil-heavy | indentation, readable |
| **Error handling** | exceptions (`try`/`except`) | return codes | `Result<T, E>`, `?` | ⏳ planned |
| **Modules / packages** | `import`, pip | `#include`, makefile/CMake | `mod`, `use`, cargo | `affer` (placeholder today) |
| **OOP** | classes, inheritance | none | impl + trait | records (`genus`), no methods |

---

## What HSD has, what it lacks

A clear-eyed inventory — useful for seeing where the language could
grow.

**Already in the design, planned to come:**
- **Tuples** — the other three have them; a natural extension.
- **Maps (dictionaries)** — one of Python's most-used types; absent
  in HSD today.
- **Error handling** — no exceptions, no `Result<T, E>`; errors are
  signalled to the caller by hand for now.
- **Generics** — the plan is *Latin inflection* as a metaprogramming
  layer: an original direction, still to be explored.
- **A full module system** — `affer` exists but is skeletal.
- **Visibility** (public/private) on declarations.

**Not currently planned:**
- **First-class functions** (passing a function as a value).
- **Closures** (functions that capture state).
- **Methods on `genus`** — HSD has records, not objects.
- **Enums with data** (like Rust's `enum` carrying values).
- **Pattern matching** (Rust's `match`, Python's `match`/`case`).
- **Sets** (Python's `set`).
- **Single characters** (C's / Rust's `char`).
- **Macros and metaprogramming** — Rust has many, C has the preprocessor.

**What HSD has and the others do not, as primitives:**
- **Actors, messages, `accipe` handlers** at the heart of the
  language.
- **`nativum` blocks** for explicit descent to a lower level.

---

## Where HSD stands

HSD takes:
- **indentation-based readability** from Python,
- **static typing with inference** from Rust,
- **translation to C for native speed**, an approach pioneered by
  Nim,
- **actors and message passing** from Erlang and Smalltalk,
- and the result is its own: a small, coherent language, not a
  Swiss-army knife.

The 27-keyword vocabulary is the most concrete sign of that choice.
HSD is not trying to be everything; it is trying to be what it is —
read like Python, run like C, checked like Rust, with actors built in
the way Erlang did them.
