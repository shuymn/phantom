/// Custom smart pointers for optimized allocation patterns
///
/// This module provides custom smart pointer implementations that optimize
/// for common allocation patterns in the Phantom codebase.
use std::alloc::{alloc, dealloc, Layout};
use std::marker::PhantomData;
use std::mem::{align_of, size_of, ManuallyDrop, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::ptr::{self, NonNull};

/// A box that stores small values inline, avoiding heap allocation
///
/// `SmallBox<T, N>` can store values up to N bytes inline. If the value
/// is larger than N bytes, it falls back to heap allocation like a regular Box.
///
/// This is particularly useful for enum types where most variants are small
/// but some might be large.
pub struct SmallBox<T, const N: usize = 64> {
    storage: SmallBoxStorage<N>,
    _phantom: PhantomData<T>,
}

#[repr(C)]
union SmallBoxStorage<const N: usize> {
    inline: MaybeUninit<[u8; N]>,
    heap: NonNull<u8>,
    _align: [u64; 0], // Ensure at least u64 alignment
}

impl<T, const N: usize> SmallBox<T, N> {
    /// Create a new SmallBox
    pub fn new(value: T) -> Self {
        let size = size_of::<T>();
        let align = align_of::<T>();

        let storage = if size <= N && align <= align_of::<u64>() {
            // Store inline
            let mut inline = MaybeUninit::<[u8; N]>::uninit();
            unsafe {
                ptr::copy_nonoverlapping(
                    &value as *const T as *const u8,
                    inline.as_mut_ptr() as *mut u8,
                    size,
                );
            }
            SmallBoxStorage { inline }
        } else {
            // Allocate on heap
            let layout = Layout::from_size_align(size, align).expect("Invalid layout");
            let ptr = unsafe { alloc(layout) };
            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }
            unsafe {
                ptr::copy_nonoverlapping(&value as *const T as *const u8, ptr, size);
            }
            SmallBoxStorage { heap: NonNull::new(ptr).expect("Allocation returned null") }
        };

        // Prevent double-drop of the original value
        std::mem::forget(value);

        SmallBox { storage, _phantom: PhantomData }
    }

    /// Check if the value is stored inline
    pub fn is_inline(&self) -> bool {
        size_of::<T>() <= N && align_of::<T>() <= align_of::<u64>()
    }

    /// Get a pointer to the stored value
    fn as_ptr(&self) -> *const T {
        if self.is_inline() {
            unsafe { &self.storage.inline as *const _ as *const T }
        } else {
            unsafe { self.storage.heap.as_ptr() as *const T }
        }
    }

    /// Get a mutable pointer to the stored value
    fn as_mut_ptr(&mut self) -> *mut T {
        if self.is_inline() {
            unsafe { &mut self.storage.inline as *mut _ as *mut T }
        } else {
            unsafe { self.storage.heap.as_ptr() as *mut T }
        }
    }
}

impl<T, const N: usize> Deref for SmallBox<T, N> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.as_ptr() }
    }
}

impl<T, const N: usize> DerefMut for SmallBox<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.as_mut_ptr() }
    }
}

impl<T, const N: usize> Drop for SmallBox<T, N> {
    fn drop(&mut self) {
        unsafe {
            // Drop the contained value
            ptr::drop_in_place(self.as_mut_ptr());

            // Free heap memory if allocated
            if !self.is_inline() {
                let layout = Layout::from_size_align(size_of::<T>(), align_of::<T>())
                    .expect("Invalid layout");
                dealloc(self.storage.heap.as_ptr(), layout);
            }
        }
    }
}

// Safety: SmallBox is Send if T is Send
unsafe impl<T: Send, const N: usize> Send for SmallBox<T, N> {}

// Safety: SmallBox is Sync if T is Sync
unsafe impl<T: Sync, const N: usize> Sync for SmallBox<T, N> {}

impl<T: Clone, const N: usize> Clone for SmallBox<T, N> {
    fn clone(&self) -> Self {
        SmallBox::new((**self).clone())
    }
}

impl<T: std::fmt::Debug, const N: usize> std::fmt::Debug for SmallBox<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SmallBox")
            .field("value", &**self)
            .field("inline", &self.is_inline())
            .finish()
    }
}

/// A vector that stores a small number of elements inline
///
/// Similar to SmallVec, but with our custom implementation optimized
/// for Phantom's use cases.
pub struct SmallVec<T, const N: usize = 4> {
    len: usize,
    storage: SmallVecStorage<T, N>,
}

union SmallVecStorage<T, const N: usize> {
    inline: ManuallyDrop<MaybeUninit<[T; N]>>,
    heap: ManuallyDrop<Vec<T>>,
}

impl<T, const N: usize> SmallVec<T, N> {
    /// Create a new empty SmallVec
    pub const fn new() -> Self {
        SmallVec {
            len: 0,
            storage: SmallVecStorage { inline: ManuallyDrop::new(MaybeUninit::uninit()) },
        }
    }

    /// Create a SmallVec with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity <= N {
            Self::new()
        } else {
            SmallVec {
                len: 0,
                storage: SmallVecStorage { heap: ManuallyDrop::new(Vec::with_capacity(capacity)) },
            }
        }
    }

    /// Check if using inline storage
    pub fn is_inline(&self) -> bool {
        self.len <= N
    }

    /// Get the current capacity
    pub fn capacity(&self) -> usize {
        if self.is_inline() {
            N
        } else {
            unsafe { (*self.storage.heap).capacity() }
        }
    }

    /// Get the length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Push an element
    pub fn push(&mut self, value: T) {
        if self.len < N {
            // Store inline
            unsafe {
                let ptr = (*self.storage.inline).as_mut_ptr() as *mut T;
                ptr.add(self.len).write(value);
            }
            self.len += 1;
        } else if self.len == N {
            // Need to spill to heap
            let mut vec = Vec::with_capacity(N * 2);
            unsafe {
                let ptr = (*self.storage.inline).as_ptr() as *const T;
                for i in 0..N {
                    vec.push(ptr.add(i).read());
                }
                vec.push(value);
                self.storage.heap = ManuallyDrop::new(vec);
            }
            self.len += 1;
        } else {
            // Already on heap
            unsafe {
                (*self.storage.heap).push(value);
            }
            self.len += 1;
        }
    }

    /// Get a slice of the elements
    pub fn as_slice(&self) -> &[T] {
        if self.is_inline() {
            unsafe {
                let ptr = (*self.storage.inline).as_ptr() as *const T;
                std::slice::from_raw_parts(ptr, self.len)
            }
        } else {
            unsafe { &(*self.storage.heap)[..] }
        }
    }

    /// Get a mutable slice of the elements
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        if self.is_inline() {
            unsafe {
                let ptr = (*self.storage.inline).as_mut_ptr() as *mut T;
                std::slice::from_raw_parts_mut(ptr, self.len)
            }
        } else {
            unsafe { &mut (*self.storage.heap)[..] }
        }
    }
}

impl<T, const N: usize> Drop for SmallVec<T, N> {
    fn drop(&mut self) {
        if self.is_inline() {
            unsafe {
                let ptr = (*self.storage.inline).as_mut_ptr() as *mut T;
                for i in 0..self.len {
                    ptr.add(i).drop_in_place();
                }
            }
        } else {
            unsafe {
                ManuallyDrop::drop(&mut self.storage.heap);
            }
        }
    }
}

impl<T, const N: usize> Deref for SmallVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for SmallVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T: Clone, const N: usize> Clone for SmallVec<T, N> {
    fn clone(&self) -> Self {
        let mut new = SmallVec::with_capacity(self.len);
        for item in self.as_slice() {
            new.push(item.clone());
        }
        new
    }
}

impl<T, const N: usize> Default for SmallVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_box_inline() {
        let sb = SmallBox::<u32, 64>::new(42);
        assert!(sb.is_inline());
        assert_eq!(*sb, 42);
    }

    #[test]
    fn test_small_box_heap() {
        let large_data = [0u8; 128];
        let sb = SmallBox::<[u8; 128], 64>::new(large_data);
        assert!(!sb.is_inline());
        assert_eq!(*sb, large_data);
    }

    #[test]
    fn test_small_box_clone() {
        let sb = SmallBox::<String, 64>::new("hello".to_string());
        let sb2 = sb.clone();
        assert_eq!(*sb, *sb2);
    }

    #[test]
    fn test_small_vec_inline() {
        let mut sv = SmallVec::<i32, 4>::new();
        sv.push(1);
        sv.push(2);
        sv.push(3);
        assert!(sv.is_inline());
        assert_eq!(sv.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_small_vec_spill_to_heap() {
        let mut sv = SmallVec::<i32, 2>::new();
        sv.push(1);
        sv.push(2);
        assert!(sv.is_inline());
        sv.push(3); // This should spill to heap
        assert!(!sv.is_inline());
        assert_eq!(sv.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_small_vec_clone() {
        let mut sv = SmallVec::<String, 2>::new();
        sv.push("hello".to_string());
        sv.push("world".to_string());
        let sv2 = sv.clone();
        assert_eq!(sv.as_slice(), sv2.as_slice());
    }
}
