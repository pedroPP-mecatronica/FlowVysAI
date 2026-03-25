//! Arena allocator — ultra-fast bump allocation for simulation data.
//!
//! # OKR 1 — KR 1.1: Zero dynamic allocations in solver loops
//!
//! All simulation buffers are allocated during `Solver::init()` via this arena.
//! The `step()` hot path operates exclusively on pre-allocated slices.
//!
//! # Performance
//! - Allocation: O(1) — pointer bump
//! - Individual deallocation: O(0) — does not exist (by design)
//! - Collective reset: O(1) — resets pointer to start
//!
//! # References
//! - PO Workflow S1: "Benchmarks de alocação < 50ns"

use bumpalo::Bump;
use std::cell::Cell;

/// Arena allocator for simulation data.
///
/// Wraps `bumpalo::Bump` with capacity tracking and cache-line aligned
/// allocation support for DOD (Data-Oriented Design) compliance.
pub struct SimArena {
    inner: Bump,
    bytes_allocated: Cell<usize>,
    capacity_bytes: usize,
}

impl SimArena {
    /// Creates a new arena with the specified pre-allocated capacity.
    ///
    /// # Sizing Guide
    /// For a 3D simulation with N cells and F scalar fields:
    /// `capacity = N × F × sizeof(f64)` + overhead
    ///
    /// Example: 10M cells, 10 fields = 10M × 10 × 8 = 800 MB
    pub fn new(capacity_bytes: usize) -> Self {
        let inner = Bump::with_capacity(capacity_bytes);
        Self {
            inner,
            bytes_allocated: Cell::new(0),
            capacity_bytes,
        }
    }

    /// Allocates a mutable slice of `count` elements initialized to `value`.
    ///
    /// # Panics
    /// Panics if the arena has insufficient space. This is intentional —
    /// arena overflow is a sizing bug that must be caught during init.
    #[inline]
    pub fn alloc_slice<T: Copy>(&self, count: usize, value: T) -> &mut [T] {
        let size = count * std::mem::size_of::<T>();
        self.bytes_allocated.set(self.bytes_allocated.get() + size);
        self.inner.alloc_slice_fill_copy(count, value)
    }

    /// Allocates a mutable slice aligned to `CACHE_LINE` (64 bytes) to avoid
    /// false sharing in parallel operations and maximize SIMD throughput.
    ///
    /// # DOD Compliance
    /// Use this for SoA field arrays that will be iterated in tight loops.
    #[inline]
    #[allow(clippy::mut_from_ref)] // Arena interior-mutability pattern: safe by construction
    pub fn alloc_aligned_slice<T: Copy>(&self, count: usize, value: T) -> &mut [T] {
        use std::alloc::Layout;
        const CACHE_LINE: usize = 64;

        let size = count * std::mem::size_of::<T>();
        let align = CACHE_LINE.max(std::mem::align_of::<T>());

        let layout =
            Layout::from_size_align(size, align).expect("Invalid layout for aligned allocation");

        let ptr = self.inner.alloc_layout(layout).as_ptr() as *mut T;

        // SAFETY: bumpalo guarantees the pointer is valid for `size` bytes.
        // We initialize all elements immediately after allocation.
        let slice = unsafe { std::slice::from_raw_parts_mut(ptr, count) };

        for elem in slice.iter_mut() {
            *elem = value;
        }

        self.bytes_allocated.set(self.bytes_allocated.get() + size);
        slice
    }

    /// Returns the total number of bytes allocated so far.
    #[inline]
    pub fn bytes_used(&self) -> usize {
        self.bytes_allocated.get()
    }

    /// Returns the configured capacity in bytes.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity_bytes
    }

    /// Returns utilization ratio (0.0 to 1.0+).
    #[inline]
    pub fn utilization(&self) -> f64 {
        self.bytes_allocated.get() as f64 / self.capacity_bytes as f64
    }

    /// Resets the entire arena. ALL previously allocated slices become invalid.
    ///
    /// Requires `&mut self` — Rust's borrow checker ensures no live references
    /// to arena-allocated data exist at this point.
    pub fn reset(&mut self) {
        self.inner.reset();
        self.bytes_allocated.set(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_alloc_and_reset() {
        let mut arena = SimArena::new(1024 * 1024); // 1 MB

        // Allocate 1000 f64s
        let slice = arena.alloc_slice(1000, 0.0_f64);
        assert_eq!(slice.len(), 1000);
        assert_eq!(arena.bytes_used(), 1000 * 8);

        // Write and verify
        for (i, v) in slice.iter_mut().enumerate() {
            *v = i as f64;
        }
        assert!((slice[999] - 999.0).abs() < f64::EPSILON);

        // Reset
        arena.reset();
        assert_eq!(arena.bytes_used(), 0);
        assert!((arena.utilization() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_arena_aligned_allocation() {
        let arena = SimArena::new(1024 * 1024);

        let slice = arena.alloc_aligned_slice(256, 0.0_f64);
        let ptr = slice.as_ptr() as usize;

        // Verify 64-byte alignment
        assert_eq!(ptr % 64, 0, "Pointer {:#x} not 64-byte aligned", ptr);
        assert_eq!(slice.len(), 256);
    }

    #[test]
    fn test_arena_utilization() {
        let arena = SimArena::new(8000); // 1000 f64s exactly
        let _ = arena.alloc_slice(500, 0.0_f64); // 4000 bytes
        assert!((arena.utilization() - 0.5).abs() < 0.01);
    }
}
