#![allow(clippy::too_many_arguments)] // for self_referencing

use crate::z3_helper::Z3Allocator;
use std::array;
use std::ops::RangeInclusive;
use crate::constraint::Constraint;

pub const SUDOKU_SIZE: usize = 9;

#[derive(Debug, Default, Eq, PartialEq, Copy, Clone, Hash)]
pub struct Cell {
    pub row: usize,
    pub col: usize,
}

impl Cell {
    pub fn new(row: usize, col: usize) -> Cell {
        Cell { row, col }
    }

    pub fn up(&self) -> Cell {
        Cell {
            row: self.row - 1,
            col: self.col,
        }
    }

    pub fn down(&self) -> Cell {
        Cell {
            row: self.row + 1,
            col: self.col,
        }
    }

    pub fn left(&self) -> Cell {
        Cell {
            row: self.row,
            col: self.col - 1,
        }
    }

    pub fn right(&self) -> Cell {
        Cell {
            row: self.row,
            col: self.col + 1,
        }
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "R{}C{}", self.row + 1, self.col + 1)
    }
}

pub struct SudokuContext<'a> {
    ctx: &'a z3::Context,
    bools: Z3Allocator<z3::ast::Bool<'a>>,
    ints: Z3Allocator<z3::ast::Int<'a>>,
    patterns: Z3Allocator<z3::Pattern<'a>>,
    sets: Z3Allocator<z3::ast::Set<'a>>,
    digits: [z3::ast::Int<'a>; 10],

    int_type: z3::Sort<'a>,

    width: usize,
    height: usize,
    digits_range: RangeInclusive<usize>,
    cells: Vec<z3::ast::Int<'a>>,

    constraints: &'a [Box<dyn Constraint + Send>],
}

impl<'a> SudokuContext<'a> {
    pub fn create(ctx: &'a z3::Context, constraints: &'a [Box<dyn Constraint + Send>]) -> Self {
        Self {
            ctx,
            bools: Z3Allocator::new(),
            ints: Z3Allocator::new(),
            patterns: Z3Allocator::new(),
            sets: Z3Allocator::new(),
            digits: array::from_fn(|i| z3::ast::Int::from_u64(ctx, i as u64)),
            int_type: z3::Sort::int(ctx),
            width: SUDOKU_SIZE,
            height: SUDOKU_SIZE,
            digits_range: 1..=SUDOKU_SIZE,
            cells:
                (0..SUDOKU_SIZE * SUDOKU_SIZE)
                    .map(|_| z3::ast::Int::fresh_const(ctx, "C"))
                    .collect()
            ,
            constraints,
        }
    }

    pub fn ctx(&self) -> &z3::Context {
        self.ctx
    }

    pub fn bools(&self) -> &Z3Allocator<z3::ast::Bool> {
        &self.bools
    }

    pub fn ints(&self) -> &Z3Allocator<z3::ast::Int> {
        &self.ints
    }

    pub fn patterns(&self) -> &Z3Allocator<z3::Pattern> {
        &self.patterns
    }

    pub fn sets(&self) -> &Z3Allocator<z3::ast::Set> {
        &self.sets
    }

    pub fn const_int(&self, n: i32) -> &z3::ast::Int {
        if (0..10).contains(&n) {
            &self.digits[n as usize]
        } else {
            self.ints
                .alloc(z3::ast::Int::from_i64(self.ctx(), n as i64))
        }
    }

    pub fn int_type(&self) -> &z3::Sort {
        &self.int_type
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
        &self.cells[col + self.width() * row]
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn digits_range(&self) -> RangeInclusive<usize> {
        self.digits_range.clone()
    }

    pub fn all_cells(&self) -> &[z3::ast::Int] {
        &self.cells
    }

    pub fn constraints(&self) -> &'a [Box<dyn Constraint + Send>] {
        self.constraints
    }
}
