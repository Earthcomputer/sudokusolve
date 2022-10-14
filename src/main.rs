#![feature(array_chunks)]
#![feature(negative_impls)]
#![feature(option_result_contains)]

mod alloc;
mod color;
mod constraint;
mod sudoku;
mod ui;

fn main() {
    ui::run();
}

pub trait DynClone<T: ?Sized> {
    fn dyn_clone(&self) -> Box<T>;
}
