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

/*
 * hsd_lege: read one line from stdin.
 *   - If `prompt` is not NULL, it is printed first (with no newline)
 *     and the output is flushed so the user sees it.
 *   - The returned string is heap-allocated. The trailing newline
 *     (and \r on Windows) is removed.
 *   - NOTE: in this phase we LEAK the buffer on each call. ARC
 *     (automatic reference counting) will manage strings later.
 *   - NOTE 2: ARC still not implemented
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

/*
 * hsd_list_num: a list of numerus (long).
 *   - `data` points to a heap-allocated array of `len` longs.
 *   - NOTE: like strings, lists currently leak. ARC will manage
 *     them later.
 */
typedef struct {
    long* data;
    long  len;
} hsd_list_num;

/*
 * hsd_numera: produce the list [a, a+1, ..., b] (b inclusive).
 *   - If b < a, returns an empty list.
 */
hsd_list_num hsd_numera(long a, long b);

#endif /* HSD_RUNTIME_H */
