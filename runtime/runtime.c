/* ============================================================
 * HSD — Hic Sunt Dracones
 * runtime.c — implementations of the support functions
 * ============================================================ */

#include "runtime.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>

/* -------- helpers -------- */

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

/* -------- hsd_lege -------- */

const char* hsd_lege(const char* prompt) {
    if (prompt != NULL) {
        fputs(prompt, stdout);
        fflush(stdout); /* show the prompt before waiting for input */
    }

    /* read into a buffer that grows as needed */
    size_t cap = 64;
    size_t len = 0;
    char* buf = (char*)malloc(cap);
    if (buf == NULL) die("out of memory in lege");

    int c;
    while ((c = fgetc(stdin)) != EOF && c != '\n') {
        if (c == '\r') continue; /* drop Windows-style \r */
        if (len + 1 >= cap) {
            cap *= 2;
            char* nb = (char*)realloc(buf, cap);
            if (nb == NULL) {
                free(buf);
                die("out of memory in lege");
            }
            buf = nb;
        }
        buf[len++] = (char)c;
    }
    buf[len] = '\0';
    return buf;
}

/* -------- numeric conversions -------- */

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

/* -------- lists -------- */

hsd_list_num hsd_numera(long a, long b) {
    hsd_list_num result;
    if (b < a) {
        result.data = NULL;
        result.len = 0;
        return result;
    }
    long count = b - a + 1;
    result.data = (long*)malloc(sizeof(long) * (size_t)count);
    if (result.data == NULL) die("out of memory in numera");
    for (long i = 0; i < count; i++) {
        result.data[i] = a + i;
    }
    result.len = count;
    return result;
}
