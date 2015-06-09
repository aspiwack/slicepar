In place data parallelism in Rust
=================================

This repository contains a toy implementation of primitives for
in-place data-parallelism using the [Rust](http://www.rust-lang.org/)
programming language and its facilities for safe mutation and
concurrency.

The library is not actually finished because it should rely on the
unsable `scoped` thread feature which would allow appropriate
lifetimes to be attached thread pools. In the current state of affairs
many arguments need to be `'static` to the point where it is actually
impossible to let jobs in a thread pool assign new jobs (it may be
possible to work around that with extra communication, I haven't tried
yet).

I've written this toy library in order to train myself with Rust. I
haven't fully developped my intuition and the code may be clumsy and
little idiomatic.

### License ###

The code in this repository is too small, inelegant and unimaginative
to be considered copyrighted. Please feel free to make any use of it.