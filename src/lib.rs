#![no_std]

// TODO: move this to ports of Martos with conditions
#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

pub mod task_manager;
pub mod timer;
