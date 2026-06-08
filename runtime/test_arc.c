/* ============================================================
 * HSD — Hic Sunt Dracones
 * test_arc.c — standalone test for ARC primitives
 *
 * Compile:
 *   Linux/macOS:  gcc test_arc.c runtime.c -I . -o test_arc
 *   Windows:      cl test_arc.c runtime.c /I .
 *
 * Run: ./test_arc (or test_arc.exe on Windows)
 *
 * Exits 0 on success, 1 on any test failure.
 * ============================================================ */

#include "runtime.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int failures = 0;

#define ASSERT(cond, msg) do { \
    if (!(cond)) { \
        fprintf(stderr, "FAIL: %s (at %s:%d)\n", msg, __FILE__, __LINE__); \
        failures++; \
    } else { \
        fprintf(stdout, "ok: %s\n", msg); \
    } \
} while (0)

int main(void) {
    /* -------- Test 1: alloc + refcount starts at 1 -------- */
    {
        char* p = (char*)hsd_arc_alloc(16);
        ASSERT(p != NULL, "alloc returns non-null");
        ASSERT(hsd_arc_refcount(p) == 1, "fresh alloc has refcount 1");
        hsd_arc_release(p);
    }

    /* -------- Test 2: retain increments, release decrements -------- */
    {
        char* p = (char*)hsd_arc_alloc(32);
        ASSERT(hsd_arc_refcount(p) == 1, "starts at 1");

        hsd_arc_retain(p);
        ASSERT(hsd_arc_refcount(p) == 2, "after one retain, count is 2");

        hsd_arc_retain(p);
        ASSERT(hsd_arc_refcount(p) == 3, "after two retains, count is 3");

        hsd_arc_release(p);
        ASSERT(hsd_arc_refcount(p) == 2, "after one release, count back to 2");

        hsd_arc_release(p);
        ASSERT(hsd_arc_refcount(p) == 1, "after another release, count is 1");

        hsd_arc_release(p);
        /* now freed; we cannot read refcount safely anymore */
    }

    /* -------- Test 3: data area is writable and intact -------- */
    {
        char* p = (char*)hsd_arc_alloc(64);
        strcpy(p, "Hello, dragons!");
        ASSERT(strcmp(p, "Hello, dragons!") == 0, "data area survives unchanged");

        hsd_arc_retain(p);
        ASSERT(strcmp(p, "Hello, dragons!") == 0, "retain doesn't disturb data");

        hsd_arc_release(p);
        ASSERT(strcmp(p, "Hello, dragons!") == 0, "release (non-final) doesn't disturb data");

        hsd_arc_release(p);
    }

    /* -------- Test 4: NULL handling -------- */
    {
        hsd_arc_retain(NULL);   /* must not crash */
        hsd_arc_release(NULL);  /* must not crash */
        ASSERT(hsd_arc_refcount(NULL) == 0, "refcount of NULL is 0");
    }

    /* -------- Test 5: many allocations + releases (stress) -------- */
    {
        const int N = 10000;
        void* ptrs[10000];
        for (int i = 0; i < N; i++) {
            ptrs[i] = hsd_arc_alloc(32);
        }
        for (int i = 0; i < N; i++) {
            ASSERT(hsd_arc_refcount(ptrs[i]) == 1, "stress: each alloc has refcount 1");
            if (failures > 0) break; /* don't spam */
        }
        for (int i = 0; i < N; i++) {
            hsd_arc_release(ptrs[i]);
        }
        printf("ok: stress test (%d alloc/release pairs) completed\n", N);
    }

    /* -------- Test 6: many retain/release on the same object -------- */
    {
        char* p = (char*)hsd_arc_alloc(8);
        const int N = 1000;
        for (int i = 0; i < N; i++) hsd_arc_retain(p);
        ASSERT(hsd_arc_refcount(p) == 1 + N, "after N retains, count is 1+N");
        for (int i = 0; i < N; i++) hsd_arc_release(p);
        ASSERT(hsd_arc_refcount(p) == 1, "after N releases, count back to 1");
        hsd_arc_release(p);
    }

    /* -------- Summary -------- */
    printf("\n");
    if (failures == 0) {
        printf("All ARC tests passed.\n");
        return 0;
    } else {
        printf("%d test(s) failed.\n", failures);
        return 1;
    }
}
