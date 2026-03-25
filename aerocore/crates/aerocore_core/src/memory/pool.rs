//! Object pool — slab-based reusable object storage with stable handles.
//!
//! # OKR 1 — KR 1.1: Zero dynamic allocations in solver loops
//!
//! Pre-allocates capacity at initialization. `acquire()` and `release()`
//! operate in O(1) without OS memory calls.
//!
//! # Use Cases
//! - Mesh entities (cells, faces, nodes)
//! - Particle tracking (LBM)
//! - Transient boundary condition objects

use slab::Slab;

/// Opaque handle to an object in the pool.
///
/// Stable across insertions/removals — safe to store in data structures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolHandle(usize);

impl PoolHandle {
    /// Returns the raw index (for advanced use, e.g., parallel arrays).
    #[inline]
    pub fn index(&self) -> usize {
        self.0
    }
}

/// Pre-allocated object pool with O(1) acquire/release.
///
/// # Memory Pooling Compliance
/// No allocation occurs after `with_capacity()`. The pool panics if
/// capacity is exceeded — this is a sizing bug, not a runtime error.
pub struct ObjectPool<T> {
    slab: Slab<T>,
    capacity: usize,
}

impl<T> ObjectPool<T> {
    /// Creates a pool with the specified pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let slab = Slab::with_capacity(capacity);
        Self { slab, capacity }
    }

    /// Acquires a slot in the pool, inserting the value.
    ///
    /// # Panics
    /// Panics if the pool is exhausted (all capacity used).
    #[inline]
    pub fn acquire(&mut self, value: T) -> PoolHandle {
        assert!(
            self.slab.len() < self.capacity,
            "ObjectPool exhausted: {} / {} slots in use. Resize at initialization.",
            self.slab.len(),
            self.capacity
        );
        PoolHandle(self.slab.insert(value))
    }

    /// Releases a slot, making it available for reuse. Returns the value.
    ///
    /// # Panics
    /// Panics if the handle is invalid (already released or out of range).
    #[inline]
    pub fn release(&mut self, handle: PoolHandle) -> T {
        self.slab.remove(handle.0)
    }

    /// Accesses an object by handle (immutable).
    #[inline]
    pub fn get(&self, handle: PoolHandle) -> Option<&T> {
        self.slab.get(handle.0)
    }

    /// Accesses an object by handle (mutable).
    #[inline]
    pub fn get_mut(&mut self, handle: PoolHandle) -> Option<&mut T> {
        self.slab.get_mut(handle.0)
    }

    /// Number of active objects in the pool.
    #[inline]
    pub fn active_count(&self) -> usize {
        self.slab.len()
    }

    /// Total capacity of the pool.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns true if the pool has no active objects.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.slab.is_empty()
    }

    /// Iterates over all active objects, yielding `(PoolHandle, &T)`.
    pub fn iter(&self) -> impl Iterator<Item = (PoolHandle, &T)> {
        self.slab.iter().map(|(k, v)| (PoolHandle(k), v))
    }

    /// Iterates mutably over all active objects.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (PoolHandle, &mut T)> {
        self.slab.iter_mut().map(|(k, v)| (PoolHandle(k), v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_acquire_release() {
        let mut pool: ObjectPool<f64> = ObjectPool::with_capacity(10);

        let h1 = pool.acquire(1.0);
        let h2 = pool.acquire(2.0);
        let h3 = pool.acquire(3.0);
        assert_eq!(pool.active_count(), 3);

        // Release middle element
        let val = pool.release(h2);
        assert!((val - 2.0).abs() < f64::EPSILON);
        assert_eq!(pool.active_count(), 2);

        // Re-acquire — should reuse the freed slot
        let h4 = pool.acquire(4.0);
        assert_eq!(pool.active_count(), 3);

        // Verify remaining values
        assert!((pool.get(h1).unwrap() - 1.0).abs() < f64::EPSILON);
        assert!((pool.get(h3).unwrap() - 3.0).abs() < f64::EPSILON);
        assert!((pool.get(h4).unwrap() - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    #[should_panic(expected = "ObjectPool exhausted")]
    fn test_pool_overflow_panics() {
        let mut pool: ObjectPool<i32> = ObjectPool::with_capacity(3);
        pool.acquire(1);
        pool.acquire(2);
        pool.acquire(3);
        pool.acquire(4); // Should panic
    }

    #[test]
    fn test_pool_iteration() {
        let mut pool: ObjectPool<&str> = ObjectPool::with_capacity(5);
        pool.acquire("pressure");
        pool.acquire("velocity");
        pool.acquire("temperature");

        let names: Vec<&str> = pool.iter().map(|(_, v)| *v).collect();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"pressure"));
        assert!(names.contains(&"velocity"));
        assert!(names.contains(&"temperature"));
    }
}
