/* ============================================================
 * HSD — Hic Sunt Dracones
 * runtime.h — support functions used by generated C code
 *
 * Every HSD program compiled to C is linked with this runtime.
 * Functions are prefixed with hsd_ to avoid clashes with user
 * symbols.
 * ============================================================ */

#ifndef HSD_RUNTIME_H
#define HSD_RUNTIME_H

#include <stddef.h>

/* ============================================================
 * ARC — Automatic Reference Counting (Phase 6a)
 *
 * Every heap-allocated value in HSD carries a reference count in
 * the 8 bytes immediately preceding the user-visible pointer.
 * The compiler emits retain/release calls automatically; the HSD
 * programmer never sees them.
 *
 * Layout of an ARC-allocated block:
 *
 *     +---------------+----------------------+
 *     | refcount (8B) | actual data (N B)    |
 *     +---------------+----------------------+
 *                     ^
 *                     User-visible pointer
 *
 * NOTE: refcounts are non-atomic in this phase. When Phase 21
 * (real actor concurrency) lands, they will become atomic.
 * Currently HSD's actors are synchronous, so no concurrent
 * access to refcounts is possible.
 *
 * NOTE: reference cycles are not handled. Cyclic data structures
 * will leak until Phase 15b introduces weak references
 * (`tenuus[T]`). See HSD-memory-model.md for the full design.
 * ============================================================ */

/*
 * hsd_arc_alloc: allocate `size` bytes for the user, plus an
 * 8-byte refcount header. The returned pointer points to the
 * user data area. The refcount is initialized to 1 (the caller
 * owns the only reference).
 *
 * Returns NULL only if the underlying malloc fails (and dies).
 */
void* hsd_arc_alloc(size_t size);

/*
 * hsd_arc_retain: increment the refcount of an ARC-tracked
 * object. Safe to call with NULL (does nothing).
 */
void hsd_arc_retain(void* ptr);

/*
 * hsd_arc_release: decrement the refcount of an ARC-tracked
 * object. If the count reaches zero, the object is freed.
 * Safe to call with NULL (does nothing).
 */
void hsd_arc_release(void* ptr);

/*
 * hsd_arc_refcount: read the current refcount of an object.
 * Intended for debugging and tests only. Not used by generated
 * code.
 */
long hsd_arc_refcount(void* ptr);

/*
 * hsd_arc_copy_str: copy a C string into ARC-allocated memory.
 * Used by the compiler to wrap string literals so all const char*
 * values in generated code are uniformly ARC-tracked.
 * The result has refcount 1.
 */
const char* hsd_arc_copy_str(const char* s);

/* ============================================================
 * I/O and conversions
 * ============================================================ */

/*
 * hsd_lege: read one line from stdin.
 *   - If `prompt` is not NULL, it is printed first (with no newline)
 *     and the output is flushed so the user sees it.
 *   - The returned string is ARC-allocated. The trailing newline
 *     (and \r on Windows) is removed.
 *   - The caller owns one reference; the compiler emits the
 *     appropriate retain/release calls.
 */
const char* hsd_lege(const char* prompt);

/*
 * hsd_numerus_ex: convert a verba into a numerus.
 *   - Accepts leading/trailing whitespace.
 *   - Aborts the program with a clear message on invalid input
 *     or out-of-range numbers.
 */
long hsd_numerus_ex(const char* s);

/*
 * hsd_realis_ex: convert a verba into a realis.
 *   - Same conventions as hsd_numerus_ex.
 */
double hsd_realis_ex(const char* s);

/* ============================================================
 * Lists
 * ============================================================ */

/*
 * hsd_list_num: a list of numerus (long).
 *   - `data` points to an ARC-allocated array of `len` longs.
 *   - The struct itself is a value type (passed by copy), but
 *     `data` is ARC-tracked. The compiler emits retain/release
 *     on `data` at the appropriate scope boundaries.
 */
typedef struct {
    long* data;
    long  len;
} hsd_list_num;

/*
 * hsd_numera: produce the list [a, a+1, ..., b] (b inclusive).
 *   - If b < a, returns an empty list (data = NULL, len = 0).
 *   - The data array is ARC-allocated with refcount 1.
 */
hsd_list_num hsd_numera(long a, long b);

/* ---- Phase 10: string helpers ---- */

/* hsd_char_at: return the character at position n as a new string. */
const char* hsd_char_at(const char* s, long n);

/* hsd_tonde: return a trimmed copy of s (no leading/trailing whitespace). */
const char* hsd_tonde(const char* s);

/* hsd_des: return 1 if s ends with suf, 0 otherwise. */
int hsd_des(const char* s, const char* suf);

/* hsd_iunge: return a new string that is the concatenation of a and b. */
const char* hsd_iunge(const char* a, const char* b);

#endif /* HSD_RUNTIME_H */

/* hsd_scinde: split s by sep, returns a list of strings as hsd_list_num
   (reusing the list struct with pointers cast to long). */
/* Note: for Phase 10 we return a simple opaque handle; full series[verba]
   support arrives in Phase 11. For now scinde is interpreter-only in C backend. */

/* hsd_forma: simple {} placeholder substitution. Variadic via sentinel. */
const char* hsd_forma_2(const char* fmt, const char* a1, const char* a2);
const char* hsd_forma_1(const char* fmt, const char* a1);
