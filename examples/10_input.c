#include <stdio.h>
#include "runtime.h"

long factorial(long n);
void principale(void);

long factorial(long n) {
    if ((n <= 1)) {
        return 1;
    } else {
        return (n * factorial((n - 1)));
    }
}

void principale(void) {
    const char* testo = hsd_lege("Inserisci un numero: ");
    long n = hsd_numerus_ex(testo);
    printf("%s%ld%s%ld\n", "Il fattoriale di ", n, " e' ", factorial(n));
}

int main(void) {
    principale();
    return 0;
}
