extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicUsize, Ordering};

pub struct Mips64Alloc {
    heap_start: AtomicUsize,
    heap_end: AtomicUsize,
    next: AtomicUsize,
}

impl Mips64Alloc {
    pub const fn new() -> Self {
        Mips64Alloc {
            heap_start: AtomicUsize::new(0),
            heap_end: AtomicUsize::new(0),
            next: AtomicUsize::new(0),
        }
    }

    pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        self.heap_start.store(heap_start, Ordering::SeqCst);
        self.heap_end
            .store(heap_start + heap_size, Ordering::SeqCst);
        self.next.store(heap_start, Ordering::SeqCst);
    }
}

unsafe impl GlobalAlloc for Mips64Alloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align();
        let size = layout.size();
        let mut current = self.next.load(Ordering::SeqCst);

        loop {
            let aligned_addr = (current + align - 1) & !(align - 1);
            let new_next = aligned_addr + size;

            // Check if the allocation fits within the heap bounds.
            if new_next > self.heap_end.load(Ordering::SeqCst) {
                return null_mut(); // Out of memory.
            }

            // Attempt to update the `next` pointer atomically.
            match self
                .next
                .compare_exchange(current, new_next, Ordering::SeqCst, Ordering::SeqCst)
            {
                Ok(_) => return aligned_addr as *mut u8,
                Err(actual) => current = actual, // Retry with the updated value.
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator doesn't reclaim memory
    }
}

#[global_allocator]
static ALLOCATOR: Mips64Alloc = Mips64Alloc::new();

/// Heap initialization.
pub fn init_heap() {
    unsafe {
        ALLOCATOR.init(0, 64 * 1024);
    }
}
