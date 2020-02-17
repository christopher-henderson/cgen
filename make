#!/usr/bin/bash
cargo clean && cargo build --release && rm -f a.out && gcc -I include/ test.c target/release/libcgen.a -pthread -ldl
