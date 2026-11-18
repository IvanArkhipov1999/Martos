use core::mem::MaybeUninit;
use esp_alloc as _;

/// Heap initialization.
/// For more information see https://github.com/esp-rs/esp-alloc/.
pub fn init_heap() {
    // Wi-Fi buffers and esp-rtos task stacks need a larger heap
    #[cfg(not(feature = "network"))]
    const HEAP_SIZE: usize = 64 * 1024;
    #[cfg(feature = "network")]
    const HEAP_SIZE: usize = 128 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        esp_alloc::HEAP.add_region(esp_alloc::HeapRegion::new(
            HEAP.as_mut_ptr() as *mut u8,
            HEAP_SIZE,
            esp_alloc::MemoryCapability::Internal.into(),
        ));
    }
}
