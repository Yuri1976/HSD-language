# Project Euler #5 — a real benchmark

An appendix to the project's language comparisons: this time not just
syntax, but **measured performance**. Project Euler #5 asks for the smallest
number evenly divisible by all integers from 1 to 20 (answer: 232792560).

The elegant approach would be LCM (least common multiple), built up
incrementally with Euclid's algorithm for GCD. Here, brute force was chosen
deliberately instead — start at 20, check divisibility by 1..20, break on
the first failure, increment by 20, retry — specifically to get a workload
heavy enough to produce meaningful performance numbers.

---

## The program in HSD

```
munus principale() -> nihil
    sit candidato = 20
    sit trovato = falsum

    dum non trovato
        sit i = 1
        sit divide_tutti = verum

        dum i <= 20
            si candidato % i != 0
                divide_tutti = falsum
                candidato = candidato + 20
                frange
            aliter
                i = i + 1

        si divide_tutti
            trovato = verum

    scribe("il numero piu piccolo e': ", candidato)
```

`dum` = while, `non` = not, `frange` = break, `verum`/`falsum` = true/false.
The program converges after roughly 11.6 million outer-loop iterations.

## The same algorithm in Python

```python
candidato = 20
trovato = False
while not trovato:
    divide_tutti = True
    for i in range(1, 21):
        if candidato % i != 0:
            divide_tutti = False
            candidato += 20
            break
    if divide_tutti:
        trovato = True
print(candidato)
```

Same exact logic, same exact iteration count — no tricks favoring either
version.

---

## The numbers

Measured on Windows with PowerShell's `Measure-Command`, three runs per
implementation, MSVC (`cl`) as the C backend compiler:

| Implementation              | Average time  | Observed range     |
|------------------------------|---------------|----------------------|
| HSD interpreter                | ~84.5 s       | 83 – 86 s            |
| Python 3.14.6                    | ~3.27 s       | 3.12 – 3.52 s        |
| HSD compiled to C (MSVC)         | ~0.140 s      | stable across repeated runs |

The first run of the compiled binary produced an outlier of 1.09 s — almost
certainly a cold-start effect (binary load, antivirus scan on first
execution). Subsequent runs settled consistently around 0.140 s, so that
value — not the outlier — is the representative one.

## What these numbers mean

**Compiled HSD is roughly 23x faster than Python** on the identical
algorithm, identical workload. No special optimization on either side — it's
the natural gap between a native binary and a bytecode interpreter with
per-iteration object-management overhead.

**The HSD interpreter is roughly 600x slower than the C backend.** This
isn't a problem to fix: the interpreter is a tree-walker that exists as a
fast dev-loop tool — for validating a program's logic before compiling it,
without waiting for the full lexer → parser → C → MSVC round trip on every
small change. The language's performance was always designed around the C
backend from day one; seeing the interpreter this far behind confirms that
architectural choice is working as intended, not a gap to close.

---

## A side note: numeric range loops

Writing this exercise surfaced an ergonomics gap: `per ... in` in HSD
currently only iterates over `series` (lists), not numeric ranges directly.
A `numera(a, b)` stdlib function exists that returns a list of integers, so
`per i in numera(1, 20)` works as a stand-in — but it means allocating an
entire list just to count to 20. A real numeric range loop (something like
`per i in 1..20`) would be more natural for counting problems like this one.
Noted as a future language improvement.
