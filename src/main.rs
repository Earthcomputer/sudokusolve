#![feature(negative_impls)]
#![feature(option_result_contains)]

mod alloc;
mod constraint;
mod sudoku;
mod ui;

fn main() {
    ui::run();
}
