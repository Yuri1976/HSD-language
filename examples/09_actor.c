#include <stdio.h>
#include <stdlib.h>
#include "runtime.h"

typedef struct Counter Counter;

struct Counter {
    long value;
};

Counter* hsd_crea_Counter(void);
void Counter_handle_Increment(Counter* self);
void Counter_handle_Show(Counter* self);
void principale(void);

void principale(void) {
    Counter* c = hsd_crea_Counter();
    Counter_handle_Increment(c);
    Counter_handle_Increment(c);
    Counter_handle_Increment(c);
    Counter_handle_Show(c);
}

Counter* hsd_crea_Counter(void) {
    Counter* self = (Counter*)malloc(sizeof(Counter));
    if (self == NULL) { fprintf(stderr, "hsd runtime: out of memory in crea Counter\n"); exit(1); }
    self->value = 0;
    return self;
}

void Counter_handle_Increment(Counter* self) {
    self->value = (self->value + 1);
}

void Counter_handle_Show(Counter* self) {
    printf("%s%ld\n", "counter value: ", self->value);
}

int main(void) {
    principale();
    return 0;
}
