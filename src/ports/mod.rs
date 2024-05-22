#[cfg(any(target_arch = "riscv32", target_arch = "xtensa"))]
pub mod xtensa_esp32;

#[cfg(not(any(target_arch = "riscv32", target_arch = "xtensa")))]
pub mod mok;
