# Martos
![Martos ci workflow](https://github.com/IvanArkhipov1999/Martos/actions/workflows/rust.yml/badge.svg)

Martos is an elegant real-time operating system designed for creating complex multi-agent systems. 
Developers have the flexibility to write software for Martos using either Rust (preferred) or C languages.

In its current version, Martos features a basic task manager and timer counter.

See Martos releases on [crates.io](https://crates.io/crates/martos).

## Programming in Rust
To develop software in Rust, you have the option to incorporate the Martos as a dependency:
```
[dependencies]
martos = "0.1.0"
```

You can explore a variety of Rust examples showcasing different architectures in the ['examples/rust-examples'](https://github.com/IvanArkhipov1999/Martos/tree/main/examples/rust-examples) directory.

## Programming in C
To develop software in C, you have the option to incorporate the Martos static library into your project:
```
target_link_libraries(target-name path-to-static-lib.a -Wl,--allow-multiple-definition)
```

You can obtain the Martos static library for supported architectures from either the release artifacts or the continuous integration (CI) artifacts.
If you wish to compile the Martos static library on your own, please refer to the ['c-library'](https://github.com/IvanArkhipov1999/Martos/tree/main/c-library) directory, 
which includes static library targets tailored for various architectures.

You can explore diverse C examples tailored for various architectures in the ['examples/c-examples'](https://github.com/IvanArkhipov1999/Martos/tree/main/examples/c-examples) directory.
