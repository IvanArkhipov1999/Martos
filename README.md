# Martos

Martos is a simple RTOS for developing multiagent realtime systems. 
Software for Martos can be written on both Rust (recommended) and C languages.

In current version it has only primitive task manager and timer counter.

## Programming on Rust
For writing software on Rust you can use Martos as dependency:
```
[dependencies]
ma_rtos = ...
```

Rust examples for different architecures you can see in examples/rust-examples.

## Programming on C
For writing software on C you can link your project with Martos static library. 
You can get Martos static library from release artifacts. 
If you want to build Martos static library yourself, see c-library directory.
It contains static library targets for different architectures.

C examples for different architecures you can see in examples/c-examples.
