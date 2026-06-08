/* ============================================================
 * HSD — Hic Sunt Dracones
 * runtime.c — implementations of the support functions
 * ============================================================ */

#include "runtime.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>

/* ============================================================
 * helpers
 * ============================================================ */

static void die(const char* msg) {
    fputs("hsd runtime: ", stderr);
    fputs(msg, stderr);
    fputc('\n', stderr);
    exit(1);
}

static void die_with_value(const char* prefix, const char* value) {
    fprintf(stderr, "hsd runtime: %s '%s'\n", prefix, value);
    exit(1);
}

/* ============================================================
 * ARC — Automatic Reference Counting (Phase 6a)
 *
 * Layout: the allocated block has an 8-byte refcount before the
 * user-visible pointer. We use a header struct so the layout is
 * explicit, but the user only ever sees the `data` part.
 *
 * Refcount lives in `header->refcount`; the pointer we return
 * to the user is `&header->data[0]`. To get from the user
 * pointer back to the header, we step back 8 bytes.
 *
 * Non-atomic for now. See note in runtime.h.
 * ============================================================ */

/* The header sits immediately before user data in memory. */
typedef struct {
    long refcount;
    /* user data follows immediately after this struct */
    char data[];
} hsd_arc_header;

/* Step back from a user pointer to its header. */
static hsd_arc_header* header_of(void* ptr) {
    return (hsd_arc_header*)((char*)ptr - offsetof(hsd_arc_header, data));
}

void* hsd_arc_alloc(size_t size) {
    /* Allocate enough for the header plus the user's requested size. */
    hsd_arc_header* h = (hsd_arc_header*)malloc(sizeof(hsd_arc_header) + size);
    if (h == NULL) {
        die("out of memory in arc_alloc");
    }
    h->refcount = 1;
    return h->data;
}

void hsd_arc_retain(void* ptr) {
    if (ptr == NULL) return;
    hsd_arc_header* h = header_of(ptr);
    h->refcount++;
}

void hsd_arc_release(void* ptr) {
    if (ptr == NULL) return;
    hsd_arc_header* h = header_of(ptr);
    h->refcount--;
    if (h->refcount == 0) {
        free(h);
    } else if (h->refcount < 0) {
        /* This means more releases than retains — a compiler bug.
         * In a debug build we want to know immediately. */
        die("arc_release: refcount went negative (compiler bug?)");
    }
}

long hsd_arc_refcount(void* ptr) {
    if (ptr == NULL) return 0;
    return header_of(ptr)->refcount;
}

const char* hsd_arc_copy_str(const char* s) {
    /* Copy a C string (typically a string literal) into ARC memory.
     * Used by the codegen to wrap string literals so all const char*
     * values are uniformly ARC-tracked. */
    if (s == NULL) return NULL;
    size_t len = strlen(s);
    char* result = (char*)hsd_arc_alloc(len + 1);
    memcpy(result, s, len + 1); /* +1 to include the trailing NUL */
    return result;
}

/* ============================================================
 * hsd_lege — read a line from stdin
 *
 * Now ARC-aware: the returned buffer is allocated through
 * hsd_arc_alloc, so the compiler can manage its lifetime with
 * retain/release calls.
 * ============================================================ */

const char* hsd_lege(const char* prompt) {
    if (prompt != NULL) {
        fputs(prompt, stdout);
        fflush(stdout); /* show the prompt before waiting for input */
    }

    /* Read into a buffer that grows as needed. We start with a
     * temporary malloc'd buffer and at the end copy into an
     * ARC-allocated one of the exact final size. This keeps the
     * ARC bookkeeping simple. */
    size_t cap = 64;
    size_t len = 0;
    char* tmp = (char*)malloc(cap);
    if (tmp == NULL) die("out of memory in lege");

    int c;
    while ((c = fgetc(stdin)) != EOF && c != '\n') {
        if (c == '\r') continue; /* drop Windows-style \r */
        if (len + 1 >= cap) {
            cap *= 2;
            char* nb = (char*)realloc(tmp, cap);
            if (nb == NULL) {
                free(tmp);
                die("out of memory in lege");
            }
            tmp = nb;
        }
        tmp[len++] = (char)c;
    }
    tmp[len] = '\0';

    /* Copy into ARC-allocated storage of the exact final size. */
    char* result = (char*)hsd_arc_alloc(len + 1);
    memcpy(result, tmp, len + 1);
    free(tmp);
    return result;
}

/* ============================================================
 * numeric conversions
 * ============================================================ */

/* Skip leading spaces and tabs (and newlines, just in case). */
static const char* skip_ws(const char* s) {
    while (*s == ' ' || *s == '\t' || *s == '\n' || *s == '\r') s++;
    return s;
}

long hsd_numerus_ex(const char* s) {
    const char* start = skip_ws(s);
    char* end;
    errno = 0;
    long n = strtol(start, &end, 10);

    if (end == start) {
        die_with_value("not a valid numerus:", s);
    }
    /* trailing whitespace is fine; anything else is not */
    end = (char*)skip_ws(end);
    if (*end != '\0') {
        die_with_value("not a valid numerus:", s);
    }
    if (errno == ERANGE) {
        die_with_value("numerus out of range:", s);
    }
    return n;
}

double hsd_realis_ex(const char* s) {
    const char* start = skip_ws(s);
    char* end;
    errno = 0;
    double d = strtod(start, &end);

    if (end == start) {
        die_with_value("not a valid realis:", s);
    }
    end = (char*)skip_ws(end);
    if (*end != '\0') {
        die_with_value("not a valid realis:", s);
    }
    if (errno == ERANGE) {
        die_with_value("realis out of range:", s);
    }
    return d;
}

/* ============================================================
 * lists
 *
 * The struct hsd_list_num is a value type. The `data` field
 * points to ARC-allocated memory.
 * ============================================================ */

hsd_list_num hsd_numera(long a, long b) {
    hsd_list_num result;
    if (b < a) {
        result.data = NULL;
        result.len = 0;
        return result;
    }
    long count = b - a + 1;
    result.data = (long*)hsd_arc_alloc(sizeof(long) * (size_t)count);
    for (long i = 0; i < count; i++) {
        result.data[i] = a + i;
    }
    result.len = count;
    return result;
}
