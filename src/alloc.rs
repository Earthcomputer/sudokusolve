use std::cell::UnsafeCell;
use std::marker::PhantomData;

pub struct Z3Allocator<T> {
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
        unsafe { &mut *self.things.get() }.push(heap as *mut T as *mut ());
        heap
    }
}

impl<T> Drop for Z3Allocator<T> {
    fn drop(&mut self) {
        for thing in self.things.get_mut() {
            unsafe {
                drop(Box::from_raw(*thing as *mut T));
            }
        }
    }
}
