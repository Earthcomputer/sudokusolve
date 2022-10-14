use std::array;
use ouroboros::self_referencing;
use crate::alloc::Z3Allocator;

pub const SUDOKU_SIZE: usize = 9;

#[derive(Debug, Default, Eq, PartialEq, Copy, Clone)]
pub struct Cell {
    pub row: usize,
    pub col: usize,
}

impl Cell {
    pub fn new(row: usize, col: usize) -> Cell {
        Cell { row, col }
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "R{}C{}", self.row + 1, self.col + 1)
    }
}

#[self_referencing(pub_extras)]
pub struct SudokuContext {
    pub ctx: z3::Context,
    #[borrows(ctx)]
    #[covariant]
    pub bools: Z3Allocator<z3::ast::Bool<'this>>,
    #[borrows(ctx)]
    #[covariant]
    ints: Z3Allocator<z3::ast::Int<'this>>,
    #[borrows(ctx)]
    #[covariant]
    digits: [z3::ast::Int<'this>; 10],

    pub width: usize,
    pub height: usize,
    #[borrows(ctx)]
    #[covariant]
    cells: Vec<z3::ast::Int<'this>>,
}

impl SudokuContext {
    pub fn create(ctx: z3::Context) -> Self {
        SudokuContextBuilder {
            ctx,
            bools_builder: |_| Z3Allocator::new(),
            ints_builder: |_| Z3Allocator::new(),
            digits_builder: |ctx| array::from_fn(|i| z3::ast::Int::from_u64(ctx, i as u64)),
            width: SUDOKU_SIZE,
            height: SUDOKU_SIZE,
            cells_builder: |ctx| (0..SUDOKU_SIZE * SUDOKU_SIZE)
                .map(|_| z3::ast::Int::fresh_const(ctx, "C"))
                .collect()
        }.build()
    }

    pub fn ctx(&self) -> &z3::Context {
        self.borrow_ctx()
    }

    pub fn bools(&self) -> &Z3Allocator<z3::ast::Bool> {
        self.borrow_bools()
    }

    pub fn ints(&self) -> &Z3Allocator<z3::ast::Int> {
        self.borrow_ints()
    }

    pub fn const_int(&self, n: i32) -> &z3::ast::Int {
        if (0..10).contains(&n) {
            self.with_digits(|digits| &digits[n as usize])
        } else {
            self.borrow_ints().alloc(z3::ast::Int::from_u64(self.ctx(), n as u64))
        }
    }

    pub fn get_cell(&self, row: usize, col: usize) -> &z3::ast::Int {
        debug_assert!(
            row < self.height(),
            "row {} is out of bounds for height {}",
            row,
            self.height()
        );
        debug_assert!(
            col < self.width(),
            "column {} is out of bounds for width {}",
            col,
            self.width()
        );
        &self.all_cells()[col + self.width() * row]
    }

    pub fn width(&self) -> usize {
        *self.borrow_width()
    }

    pub fn height(&self) -> usize {
        *self.borrow_height()
    }

    pub fn all_cells(&self) -> &[z3::ast::Int] {
        self.borrow_cells()
    }
}
