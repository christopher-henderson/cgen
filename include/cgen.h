#ifndef _RUST_CGEN_BINDINGS_H
#define _RUST_CGEN_BINDINGS_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

//typedef struct Thing {
//    void (*free)(struct Thing *thing);
//} Thing;

typedef struct RustBuffer_c_uchar {
  unsigned char *data;
  uintptr_t len;
} RustBuffer_c_uchar;

/**
 * A RustString wraps a RustBuffer<c_uchar>.
 *
 * RustString's str_len is the length of the string WITHOUT the null byte.
 * To get the length of the entire buffer, including the null byte, access
 * the internal buffer's own len field.
 */
typedef struct RustString {
  RustBuffer_c_uchar buf;
  uintptr_t str_len;
  void (*free)(struct RustString *ptr);
} RustString;

void rustfree(void *ptr);

RustString *ruststringnew(uintptr_t len);



#endif /* _RUST_CGEN_BINDINGS_H */
