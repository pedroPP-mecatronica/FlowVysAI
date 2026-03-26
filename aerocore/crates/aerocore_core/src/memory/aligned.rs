//! Aligned vector — heap-allocated storage with configurable alignment.
//!
//! # OKR 1 — KR 2.2: 90% Cache Hit Rate via DOD
//!
//! Provides a `Vec`-like container where the underlying buffer is aligned
//! to cache line boundaries (64 bytes), enabling optimal SIMD vectorization
//! and avoiding false sharing in parallel solver loops.
//!
//! # Design Decision
//! While `SimArena` is preferred for hot-path data, `AlignedVec` is useful
//! for persistent, resizable storage (e.g., mesh data loaded from files).

use std::alloc::{self, Layout};
use std::ops::{Deref, DerefMut};

/// Default cache line size on modern x86/ARM processors.
pub const CACHE_LINE_SIZE: usize = 64;

/// Heap-allocated vector with guaranteed alignment.
///
/// The backing memory is always aligned to at least `ALIGN` bytes.
/// Implements `Deref<Target=[T]>` for seamless slice access.
pub struct AlignedVec<T, const ALIGN: usize = CACHE_LINE_SIZE> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

// SAFETY: AlignedVec owns its data and T: Send implies AlignedVec: Send
unsafe impl<T: Send, const ALIGN: usize> Send for AlignedVec<T, ALIGN> {}
// SAFETY: AlignedVec behind &ref only exposes &[T], which is Sync if T: Sync
unsafe impl<T: Sync, const ALIGN: usize> Sync for AlignedVec<T, ALIGN> {}

impl<T: Copy, const ALIGN: usize> AlignedVec<T, ALIGN> {
    /// Creates a new aligned vector with the given capacity, all elements
    /// initialized to `value`.
    ///
    /// # Panics
    /// Panics if the allocation fails or if `ALIGN` is not a power of two.
    pub fn new(capacity: usize, value: T) -> Self {
        assert!(ALIGN.is_power_of_two(), "Alignment must be power of two");
        assert!(
            ALIGN >= std::mem::align_of::<T>(),
            "Alignment must be >= natural alignment of T"
        );

        if capacity == 0 {
            return Self {
                ptr: std::ptr::NonNull::dangling().as_ptr(),
                len: 0,
                capacity: 0,
            };
        }

        let layout = Self::make_layout(capacity);
        // SAFETY: layout has non-zero size (capacity > 0) and valid alignment
        let ptr = unsafe { alloc::alloc(layout) as *mut T };
        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }

        // Initialize all elements
        // SAFETY: ptr is valid for `capacity` elements, all will be initialized
        unsafe {
            for i in 0..capacity {
                std::ptr::write(ptr.add(i), value);
            }
        }

        Self {
            ptr,
            len: capacity,
            capacity,
        }
    }

    /// Creates an aligned vector from existing data, copying elements.
    pub fn from_slice(data: &[T]) -> Self
    where
        T: Default,
    {
        if data.is_empty() {
            return Self::new(0, unsafe { std::mem::zeroed() });
        }

        let vec = Self::new(data.len(), data[0]);
        // SAFETY: both src and dst are valid for data.len() elements
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), vec.ptr, data.len());
        }
        vec
    }

    fn make_layout(capacity: usize) -> Layout {
        let size = capacity * std::mem::size_of::<T>();
        let align = ALIGN.max(std::mem::align_of::<T>());
        Layout::from_size_align(size, align).expect("Invalid layout for AlignedVec")
    }

    /// Returns a raw pointer to the underlying data (for SIMD / GPU upload).
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    /// Returns a mutable raw pointer to the underlying data.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }

    /// Returns the number of elements.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the vector is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the alignment in bytes.
    #[inline]
    pub fn alignment(&self) -> usize {
        ALIGN
    }

    /// Fills all elements with the given value (no allocation).
    #[inline]
    pub fn fill(&mut self, value: T) {
        for i in 0..self.len {
            // SAFETY: i < self.len, which was valid at allocation
            unsafe {
                std::ptr::write(self.ptr.add(i), value);
            }
        }
    }
}

impl<T, const ALIGN: usize> Deref for AlignedVec<T, ALIGN> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &[T] {
        if self.capacity == 0 {
            return &[];
        }
        // SAFETY: ptr is valid for self.len elements, all initialized
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl<T, const ALIGN: usize> DerefMut for AlignedVec<T, ALIGN> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [T] {
        if self.capacity == 0 {
            return &mut [];
        }
        // SAFETY: ptr is valid for self.len elements, all initialized
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl<T, const ALIGN: usize> Drop for AlignedVec<T, ALIGN> {
    fn drop(&mut self) {
        if self.capacity > 0 {
            let layout = Layout::from_size_align(
                self.capacity * std::mem::size_of::<T>(),
                ALIGN.max(std::mem::align_of::<T>()),
            )
            .expect("Invalid layout during dealloc");
            // SAFETY: ptr was allocated with this exact layout
            unsafe {
                alloc::dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aligned_vec_basic() {
        let vec: AlignedVec<f64> = AlignedVec::new(1000, 0.0);
        assert_eq!(vec.len(), 1000);
        assert!((vec[0] - 0.0).abs() < f64::EPSILON);

        // Verify alignment
        let ptr = vec.as_ptr() as usize;
        assert_eq!(ptr % CACHE_LINE_SIZE, 0, "Not cache-line aligned");
    }

    #[test]
    fn test_aligned_vec_write_read() {
        let mut vec: AlignedVec<f64> = AlignedVec::new(100, 0.0);
        for i in 0..100 {
            vec[i] = i as f64 * 1.5;
        }
        assert!((vec[99] - 148.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_aligned_vec_fill() {
        let mut vec: AlignedVec<f64> = AlignedVec::new(50, 0.0);
        vec.fill(42.0);
        for v in vec.iter() {
            assert!((v - 42.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_aligned_vec_empty() {
        let vec: AlignedVec<f64> = AlignedVec::new(0, 0.0);
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
    }
}
