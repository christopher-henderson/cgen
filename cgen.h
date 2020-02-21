#ifndef _RUST_CGEN_BINDINGS_H
#define _RUST_CGEN_BINDINGS_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct RustBuffer_c_char {
  char *data;
  uintptr_t len;
  uintptr_t capacity;
  void (*free)(struct RustBuffer_c_char *ptr);
  void (*push)(struct RustBuffer_c_char *ptr, char value);
} RustBuffer_c_char;

typedef RustBuffer_c_char RustCharBuffer;
typedef RustBuffer_c_char RustString;

typedef struct RustBuffer_f64 {
  double *data;
  uintptr_t len;
  uintptr_t capacity;
  void (*free)(struct RustBuffer_f64 *ptr);
} RustBuffer_f64;

typedef RustBuffer_f64 RustDoubleBuffer;

typedef struct RustBuffer_f32 {
  float *data;
  uintptr_t len;
  uintptr_t capacity;
  void (*free)(struct RustBuffer_f32 *ptr);
} RustBuffer_f32;

typedef RustBuffer_f32 RustFloatBuffer;

typedef struct RustBuffer_c_int {
  int *data;
  uintptr_t len;
  uintptr_t capacity;
  void (*free)(struct RustBuffer_c_int *ptr);
} RustBuffer_c_int;

typedef RustBuffer_c_int RustIntBuffer;

typedef struct RustBuffer_c_long {
  long *data;
  uintptr_t len;
  uintptr_t capacity;
  void (*free)(struct RustBuffer_c_long *ptr);
} RustBuffer_c_long;

typedef RustBuffer_c_long RustLongBuffer;

typedef struct RustBuffer_c_short {
  short *data;
  uintptr_t len;
  uintptr_t capacity;
  void (*free)(struct RustBuffer_c_short *ptr);
} RustBuffer_c_short;

typedef RustBuffer_c_short RustShortBuffer;

typedef struct RustBuffer_c_uchar {
  unsigned char *data;
  uintptr_t len;
  uintptr_t capacity;
  void (*free)(struct RustBuffer_c_uchar *ptr);
} RustBuffer_c_uchar;

typedef RustBuffer_c_uchar RustUcharBuffer;

typedef struct RustBuffer_c_uint {
  unsigned int *data;
  uintptr_t len;
  uintptr_t capacity;
  void (*free)(struct RustBuffer_c_uint *ptr);
} RustBuffer_c_uint;

typedef RustBuffer_c_uint RustUintBuffer;

typedef struct RustBuffer_c_ulong {
  unsigned long *data;
  uintptr_t len;
  uintptr_t capacity;
  void (*free)(struct RustBuffer_c_ulong *ptr);
} RustBuffer_c_ulong;

typedef RustBuffer_c_ulong RustUlongBuffer;

typedef struct RustBuffer_c_ushort {
  unsigned short *data;
  uintptr_t len;
  uintptr_t capacity;
  void (*free)(struct RustBuffer_c_ushort *ptr);
} RustBuffer_c_ushort;

typedef RustBuffer_c_ushort RustUshortBuffer;

RustCharBuffer *new_rust_char_buffer(size_t capacity);

RustDoubleBuffer *new_rust_double_buffer(size_t capacity);

RustFloatBuffer *new_rust_float_buffer(size_t capacity);

RustIntBuffer *new_rust_int_buffer(size_t capacity);

RustLongBuffer *new_rust_long_buffer(size_t capacity);

RustShortBuffer *new_rust_short_buffer(size_t capacity);

RustString *new_rust_string(size_t capacity);

RustUcharBuffer *new_rust_uchar_buffer(size_t capacity);

RustUintBuffer *new_rust_uint_buffer(size_t capacity);

RustUlongBuffer *new_rust_ulong_buffer(size_t capacity);

RustUshortBuffer *new_rust_ushort_buffer(size_t capacity);

#endif /* _RUST_CGEN_BINDINGS_H */
