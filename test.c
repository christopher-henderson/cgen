#include <cgen.h>
#include <unistd.h>
#include <stdio.h>
#include <string.h>

int main() {

    for (int i=0; i<9; i++) {
        RustCharBuffer * string = new_rust_char_buffer(6);
//        *(string->data) = 'h';
//        *(string->data + 1) = '\0';
//        strcpy(string->data, "hello");
        string->push_all(string, "hello", 6);
        string->push_all(string, " world", 7);
//        push(string, 'h');
//        push(string, 'e');
//        push(string, 'l');
//        push(string, 'l');
//        push(string, 'o');
//        push(string, '\0');
//        string->push(string, 'h');
//        string->push(string, 'e');
//        string->push(string, 'l');
//        string->push(string, 'l');
//        string->push(string, 'o');
//        string->push(string, '\0');
        printf("%s\n", string->data);
        string->free(string);
    }


//    int index = 0;
//    for (char* i=string->data;;i++) {
//        printf("%d\n", index);
//        *i = 1;
//        index ++;
//    }
//    strcpy(string->data, "Hello, World!");
//    printf("%s\n", string->data);

}