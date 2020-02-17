#include <cgen.h>
#include <unistd.h>
#include <stdio.h>
#include <string.h>

int main() {
    RustString * string = ruststringnew(12);
    strcpy(string->buf.data, "Hello, World!");
    printf("%s\n", string->buf.data);
    string->free(string);
}