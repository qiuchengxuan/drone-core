use core::{
    alloc::Layout,
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

#[derive(Copy, Clone, Default)]
pub struct Statistics {
    pub block_size: usize,
    pub capacity: usize,
    pub remain: usize,
}

/// The set of free memory blocks.
///
/// It operates by connecting unallocated regions of memory together in a linked
/// list, using the first word of each unallocated region as a pointer to the
/// next.
pub struct Pool {
    /// Total blocks
    capacity: usize,
    /// Remain blocks
    remain: AtomicUsize,
    /// Block size. Doesn't change in the run-time.
    block_size: usize,
    /// Address of the byte past the last element. Doesn't change in the
    /// run-time.
    edge: *mut u8,
    /// Free List of previously allocated blocks.
    free: AtomicPtr<u8>,
    /// Pointer growing from the starting address until it reaches the `edge`.
    uninit: AtomicPtr<u8>,
}

unsafe impl Sync for Pool {}

impl Pool {
    /// Creates a new `Pool`.
    pub const fn new(address: usize, block_size: usize, capacity: usize) -> Self {
        Self {
            capacity,
            remain: AtomicUsize::new(capacity),
            block_size,
            edge: (address + block_size * capacity) as *mut u8,
            free: AtomicPtr::new(ptr::null_mut()),
            uninit: AtomicPtr::new(address as *mut u8),
        }
    }

    /// Returns capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the block size.
    #[inline]
    pub fn block_size(&self) -> usize {
        self.block_size
    }

    /// Returns pool allocation statistics.
    pub fn statistics(&self) -> Statistics {
        Statistics {
            block_size: self.block_size,
            capacity: self.capacity,
            remain: self.remain.load(Ordering::Relaxed),
        }
    }

    /// Allocates one block of memory.
    ///
    /// If this method returns `Some(addr)`, then the `addr` returned will be
    /// non-null address pointing to the block. If this method returns `None`,
    /// then the pool is exhausted.
    ///
    /// This operation is lock-free and has *O(1)* time complexity.
    pub fn allocate(&self) -> Option<NonNull<u8>> {
        unsafe { self.alloc_free().or_else(|| self.alloc_uninit()) }
    }

    /// Deallocates the block referenced by `ptr`.
    ///
    /// This operation is lock-free and has *O(1)* time complexity.
    ///
    /// # Safety
    ///
    /// * `ptr` must point to a block previously allocated by
    ///   [`alloc`](Pool::alloc).
    /// * `ptr` must not be used after deallocation.
    #[allow(clippy::cast_ptr_alignment)]
    pub unsafe fn deallocate(&self, ptr: NonNull<u8>) {
        loop {
            let curr = self.free.load(Ordering::Acquire);
            unsafe { ptr::write(ptr.as_ptr().cast::<*mut u8>(), curr) };
            let next = ptr.as_ptr().cast::<u8>();
            if self
                .free
                .compare_exchange_weak(curr, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                self.remain.fetch_add(1, Ordering::Relaxed);
                break;
            }
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    unsafe fn alloc_free(&self) -> Option<NonNull<u8>> {
        loop {
            let curr = self.free.load(Ordering::Acquire);
            if curr.is_null() {
                break None;
            }
            let next = unsafe { ptr::read(curr as *const *mut u8) };
            if self
                .free
                .compare_exchange_weak(curr, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                self.remain.fetch_sub(1, Ordering::Relaxed);
                break Some(unsafe { NonNull::new_unchecked(curr) });
            }
        }
    }

    unsafe fn alloc_uninit(&self) -> Option<NonNull<u8>> {
        loop {
            let curr = self.uninit.load(Ordering::Relaxed);
            if curr == self.edge {
                break None;
            }
            let next = unsafe { curr.add(self.block_size) };
            if self
                .uninit
                .compare_exchange_weak(curr, next, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                self.remain.fetch_sub(1, Ordering::Relaxed);
                break Some(unsafe { NonNull::new_unchecked(curr) });
            }
        }
    }
}

pub trait Fits: Copy {
    fn fits(self, pool: &Pool) -> bool;
}

impl<'a> Fits for &'a Layout {
    #[inline]
    fn fits(self, pool: &Pool) -> bool {
        self.size() <= pool.block_size
    }
}

impl Fits for NonNull<u8> {
    #[inline]
    fn fits(self, pool: &Pool) -> bool {
        (self.as_ptr().cast::<u8>()) < pool.edge
    }
}
