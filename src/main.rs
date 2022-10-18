#![allow(incomplete_features)] // for trait_upcasting

#![feature(array_chunks)]
#![feature(array_windows)]
#![feature(negative_impls)]
#![feature(option_result_contains)]
#![feature(trait_upcasting)]

mod z3_helper;
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
