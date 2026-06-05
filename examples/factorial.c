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
    long result = factorial(5);
    printf("%s%ld\n", "Factorial of 5: ", result);
}

int main(void) {
    principale();
    return 0;
}
