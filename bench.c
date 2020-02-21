#include <unistd.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

int main() {
    for (int i=0; i<999999999; i++) {
        char * str= (char *) malloc(sizeof(char) * 6);
        strcpy(str, "hello");
//        *str = 'h';
//        *(str + 1) = 'e';
//        *(str + 2) = 'l';
//        *(str + 3) = 'l';
//        *(str + 4) = '0';
//        *(str + 5) = '\0';
        free(str);
    }

}