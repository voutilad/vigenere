# The Vigenere Cipher

This is one of the exercises from the [Coursera Cryptography](https://www.coursera.org/learn/cryptography/) course from the University of Maryland, College Park.

General premise is an example of heuristic approaches to cracking a usage of the Vigenere cipher in two parts:

1. Guessing the keylength
2. Using heuristics to quickly brute force viable keys of a given keylength

# Running the code
The code was developed on [OpenBSD](http://www.openbsd.org)-current, but uses primarily just the Rust standard library and the [rand](https://crates.io/crates/rand) crate.

1. Build using cargo:

```bash
cargo build --release
```

2. Generate some possible key lengths and their weights:

```bash
./target/release/vigenere key input/cipher.txt`
```

3. Choose a key length (say _n_) and try brute forcing to generate plaintext:

```bash
./target/release/vigenere crack [n] input/cipher.txt
```

# Notes on the implementation
I took some liberties in how the problem is approached, mainly:

1. Keys are of some finite size, mainly only up to _32 bytes_ in length.

2. The frequency of the english lowercase `e` is assumed to be ~0.9 based on a manual analysis of [Moby Dick](./input/moby_dick.txt). (This was done in Python's REPL and I don't offer any code for this claim at the moment.)

3. A random search of the probable keyspace is used. (Hence using the rand crate's uniform distributions to basically "roll dice" to pick candidate byte values.)

4. Generated plaintext is considered "viable" and printed to stdout if:
   a. all bytes are in the printable ASCII range (0x20 - 0x7f)
   b. the tokens `the` and `and` are found, surrounded by whitespace
