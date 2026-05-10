// expect: no-unsafe-fn, no-goto
// expect: todo-comment

// TODO: fix
#include <stdio.h>
#include <string.h>

void main() {
    char buf[10];
    gets(buf);
    strcpy(buf, "hello");
    goto end;
end:
    if (1) {}
    return;
}
