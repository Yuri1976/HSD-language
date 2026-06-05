# HSD — Grammar (EBNF)

> The formal grammar of **HSD** (*Hic Sunt Dracones*): a compiled,
> multi-paradigm language with indentation-based syntax, static typing
> with inference, Latin keywords, and a first-class actor model for
> concurrency.

---

## About this project

HSD — short for *Hic Sunt Dracones*, the Latin phrase that
cartographers used to write across the unmapped regions of their maps —
is a personal project that began more than five years ago. Like many
projects of that kind, it spent a long time in a drawer.

The first version was written in C. It was educational in the worst
sense of the word: I learned what it feels like when memory management
gets ahead of you. Function pointers, hand-rolled ownership conventions,
half-written cleanup paths — the kind of code that compiles, even runs,
but whose author no longer trusts it. After a while I put it down.

I came back to HSD recently and rewrote most of the compiler in Rust.
The borrow checker, after the years away, was a relief: for the first
time I could move pieces around without losing track of who owned what —
which, for a project that's essentially *about* ownership and memory,
made the difference. The C parts that survived are the runtime — the
small library of support functions that compiled HSD programs link
against.

The project today is in a **didactic state**: it works end-to-end. A
`.hsd` source file is lexed, parsed, type-checked, and then either
executed by a tree-walking interpreter or compiled to a native
executable through a C backend. None of this is fast yet, none of it is
production-ready, and several promises in the original design (such as
the ARC memory model) are documented but not yet built. But it runs.
The dragons are a little more mapped than they were five years ago.

This document is the grammar of the language as it stands today.

---

## The language at a glance

- **Indentation-based syntax** (like Python), no curly braces.
- **Static types with inference** (like Rust): required on function
  signatures, inferred on local variables.
- **Latin keywords** — `munus`, `redde`, `si`, `aliter`, `sit`, `fixum`,
  and others — for a coherent linguistic identity.
- **Compiles to native code via C** (the Nim model): a `.hsd` file
  becomes a standalone executable.
- **Actors as a language primitive**: concurrency lives in the core,
  not in a library.
- **A small keyword set** (27 in total): the standard library grows
  without weighing down the grammar.

Functions like `scribe`, `lege`, `numera` are *not* keywords: they are
standard-library functions, recognized by the semantic analyzer as
pre-declared names.

---

## Notation

| Symbol      | Meaning                            |
|-------------|------------------------------------|
| `=`         | defines a rule                     |
| `;`         | end of a rule                      |
| `|`         | alternative ("or")                 |
| `{ ... }`   | zero or more repetitions           |
| `[ ... ]`   | optional                           |
| `( ... )`   | grouping                           |
| `" "`       | literal text (the actual symbol)   |
| `(* ... *)` | comment                            |
| UPPERCASE   | terminal produced by the lexer     |

---

## 1. Top-level structure

```ebnf
program        = { NEWLINE }, { top_declaration } ;

top_declaration =
      import
    | function_def
    | genus_def
    | actor_def
    | nuntius_def
    | declaration ;
```

---

## 2. Module import

```ebnf
import = "affer", VERBA, NEWLINE ;
   (* affer "mathematica" *)
```

---

## 3. Indented blocks

Blocks do not use curly braces: the lexer emits the terminals `INDENT`
and `DEDENT`, much like Python.

```ebnf
block = NEWLINE, INDENT, statement, { statement }, DEDENT ;
```

---

## 4. Name declaration

```ebnf
declaration = ( "sit" | "fixum" ), IDENT, [ ":", type ],
              "=", expression, NEWLINE ;
   (* sit x = 5                -> variable, type inferred  *)
   (* fixum pi: realis = 3.14  -> constant, type explicit  *)
```

---

## 5. Types

```ebnf
type = IDENT, [ "[", type, "]" ] ;
   (* numerus | realis | veritas | verba | series[numerus] *)
```

---

## 6. Functions

```ebnf
function_def = "munus", IDENT, "(", [ parameters ], ")",
               [ "->", type ], block ;

parameters   = parameter, { ",", parameter } ;
parameter    = IDENT, ":", type ;
```

---

## 7. Records (genus)

```ebnf
genus_def = "genus", IDENT, NEWLINE, INDENT, field, { field }, DEDENT ;
field     = IDENT, ":", type, NEWLINE ;
```

---

## 8. Statements

```ebnf
statement =
      declaration
    | conditional
    | while_loop
    | for_loop
    | return_stmt
    | send_stmt
    | nativum_block
    | "frange", NEWLINE
    | "perge", NEWLINE
    | expression, NEWLINE ;

conditional = "si", expression, block,
              { "aliter", "si", expression, block },
              [ "aliter", block ] ;

while_loop  = "dum", expression, block ;

for_loop    = "per", IDENT, "in", expression, block ;

return_stmt = "redde", [ expression ], NEWLINE ;
```

---

## 9. Actors (the concurrency model)

```ebnf
actor_def        = "actor", IDENT, NEWLINE, INDENT,
                   field_or_handler, { field_or_handler }, DEDENT ;

field_or_handler = field | handler ;

handler          = "accipe", IDENT, [ "(", [ parameters ], ")" ], block ;
   (* accipe = "receive": handles one kind of message *)

nuntius_def      = "nuntius", IDENT, [ "(", [ parameters ], ")" ], NEWLINE ;
   (* nuntius = "message": declares a message type *)

send_stmt        = "mitte", expression, "ad", expression, NEWLINE ;
   (* mitte Incrementa ad contator *)
```

---

## 10. Low-level opt-out

```ebnf
nativum_block = "nativum", block ;
   (* a region of shared/manual memory: fast, but at your own risk.
      Outside the guarantees of the actor model. *)
```

---

## 11. Expressions — precedence cascade

The order of the rules **encodes precedence**: what is further down
"binds more tightly".

```ebnf
expression = expr_or ;

expr_or    = expr_and,  { "vel", expr_and } ;
expr_and   = expr_not,  { "et",  expr_not } ;
expr_not   = [ "non" ], comparison ;
comparison = sum, { ( "==" | "!=" | "<" | ">" | "<=" | ">=" ), sum } ;
sum        = term,    { ( "+" | "-" ), term } ;
term       = unary,   { ( "*" | "/" | "%" ), unary } ;
unary      = [ "-" ], postfix ;
postfix    = primary, { call | index | access } ;

call       = "(", [ arguments ], ")" ;
index      = "[", expression, "]" ;
access     = ".", IDENT ;
arguments  = expression, { ",", expression } ;

primary =
      NUMERUS
    | REALIS
    | VERBA
    | "verum" | "falsum" | "nihil"
    | "ipse"
    | IDENT
    | "crea", IDENT, [ "(", [ arguments ], ")" ]
    | list
    | "(", expression, ")" ;

list = "[", [ arguments ], "]" ;
```

---

## 12. Lexer tokens

**Keywords** (27 in total): `sit` `fixum` `munus` `redde` `si` `aliter`
`dum` `per` `in` `frange` `perge` `verum` `falsum` `et` `vel` `non`
`nihil` `genus` `affer` `actor` `accipe` `nuntius` `mitte` `ad` `crea`
`ipse` `nativum`

**Literals:** `NUMERUS` (integer) · `REALIS` (decimal) · `VERBA` (string)
· `IDENT` (identifier)

**Operators:** `+` `-` `*` `/` `%` `=` `==` `!=` `<` `>` `<=` `>=` `->`

**Punctuation:** `(` `)` `[` `]` `,` `:` `.`

**Layout:** `NEWLINE` · `INDENT` · `DEDENT` · `EOF`

> Type names (`numerus`, `realis`, `veritas`, `verba`, `series`) and
> standard-library names (`scribe`, `lege`, `numera`, etc.) are **not**
> keywords. The lexer sees them as `IDENT`s; the semantic analyzer
> recognizes them as pre-declared names. This keeps the grammatical core
> small and lets the standard library grow without touching the lexer
> or parser.

---

## 13. Example programs

### Basic example

```romanes
munus principale() -> nihil
    fixum nomen = "Roma"
    sit cives = 1000000
    si cives > 500000
        scribe(nomen, " est urbs magna")
    aliter
        scribe(nomen, " est urbs parva")
```

### With actors

```romanes
# Messages the actor can receive
nuntius Incrementa
nuntius Monstra

actor Contator
    sit valor: numerus = 0

    accipe Incrementa
        valor = valor + 1

    accipe Monstra
        scribe("Valor: ", valor)

munus principale() -> nihil
    sit c = crea Contator
    mitte Incrementa ad c
    mitte Incrementa ad c
    mitte Monstra ad c
```

### With the low-level opt-out

The example below uses *illustrative* function names
(`alloca_nativum`, `libera_nativum`): it shows the `nativum` construct
in use, but does not commit the standard library to those names.

```romanes
munus computa_intense() -> nihil
    nativum
        # direct shared memory: fast, but at your own risk
        sit buffer = alloca_nativum(1024)
        # ... real-time critical loop ...
        libera_nativum(buffer)
```

---

## Design notes

**Small core, large library.** HSD has 27 keywords — among the leanest
language cores around (Go has 25, Lua 22, Python 35, C around 44, Rust
over 50). The choice is deliberate: anything that doesn't introduce a
new *grammatical form* belongs in the standard library. Adding a
function like `lege` or `scribe` requires no change to the lexer or
parser.

**Latin keywords.** Not a folkloric choice. Latin offers a rich technical
vocabulary that is free from the implications and baggage of programming
languages past (unlike `if`, `class`, `return`, which arrive already
*loaded* with the meanings of every language that uses them). It also
lets HSD name its own constructs precisely — `accipe` for "receive a
message", `mitte` for "send", `ipse` for "self" — without colliding with
any existing language.

**Actors, not objects.** HSD distinguishes `genus` (records: passive
containers of data) from `actor` (active objects with state and
behavior). This separates the two concepts cleanly: data with no
identity, and identities that respond to messages. The philosophy is
closer to Erlang than to Java.

**Opt-out, not opt-in.** The guarantees of the language (automatic
memory management, the actor model) are the *default*. The `nativum`
block is the explicit exit, for code that needs manual control and
accepts the responsibility that comes with it. This is the opposite of
the C model, where danger is implicit.
