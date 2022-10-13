use std::cell::UnsafeCell;
use std::marker::PhantomData;

pub struct Z3Allocator<T> {
    // This type is covariant, but without erased pointers the compiler doesn't detect it as such.
    // Covariance is important for self-referential structs that may contain this allocator.
    // We also use a cell because &mut Z3Allocation<T> is invariant.
    things: UnsafeCell<Vec<*mut ()>>,
    _phantom: PhantomData<T>,
}

impl<T> !Send for Z3Allocator<T> {}
impl<T> !Sync for Z3Allocator<T> {}

impl<T> Z3Allocator<T> {
    pub fn new() -> Self {
        Self {
            things: UnsafeCell::new(Vec::new()),
            _phantom: PhantomData,
        }
    }

    pub fn alloc(&self, thing: T) -> &T {
        let heap = Box::leak(Box::new(thing));
        // Safety: we are !Send and !Sync so there are guaranteed to be no data races.
        unsafe { &mut *self.things.get() }.push(heap as *mut T as *mut ());
        heap
    }
}

impl<T> Drop for Z3Allocator<T> {
    fn drop(&mut self) {
        for thing in self.things.get_mut() {
            // Safety: These pointers come from Box::leak in the alloc method.
            unsafe {
                drop(Box::from_raw(*thing as *mut T));
            }
        }
    }
}
