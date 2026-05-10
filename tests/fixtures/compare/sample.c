// Realistic C code with various issues
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void main() {
    char buf[64];
    gets(buf);
    strcpy(buf, "hello world this is a long string");

    char *ptr = malloc(100);
    printf("allocated: %p\n", ptr);

    int x = 42;
    if (sizeof(ptr) > 4) {
        printf("64-bit\n");
    }

    goto cleanup;

    printf("unreachable\n");

cleanup:
    if (x > 0) {
    }

    free(ptr);
}
