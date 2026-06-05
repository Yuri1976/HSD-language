#include <stdio.h>
#include "runtime.h"

void principale(void);

void principale(void) {
    long sum = 0;
    hsd_list_num _hsd_list_0 = hsd_numera(1, 10);
    for (long _hsd_idx_0 = 0; _hsd_idx_0 < _hsd_list_0.len; _hsd_idx_0++) {
        long i = _hsd_list_0.data[_hsd_idx_0];
        sum = (sum + i);
    }
    printf("%s%ld\n", "La somma da 1 a 10 e' ", sum);
}

int main(void) {
    principale();
    return 0;
}
