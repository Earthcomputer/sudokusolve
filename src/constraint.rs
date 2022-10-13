use crate::sudoku::{SudokuContext, SUDOKU_SIZE};
use z3::ast::Ast;

pub trait Constraint {
    fn apply<'a>(&self, solver: &z3::Solver, context: &'a SudokuContext);
}

pub struct DigitsConstraint;
impl Constraint for DigitsConstraint {
    fn apply<'a>(&self, solver: &z3::Solver, context: &'a SudokuContext) {
        for cell in context.all_cells() {
            solver.assert(
                context.bools().alloc(z3::ast::Bool::or(
                    context.ctx(),
                    &(1..=9)
                        .map(|i| context.bools().alloc(cell._eq(context.const_int(i))))
                        .collect::<Vec<_>>(),
                )),
            );
        }
    }
}

pub struct LatinSquareConstraint;
impl Constraint for LatinSquareConstraint {
    fn apply<'a>(&self, solver: &z3::Solver, context: &'a SudokuContext) {
        for row in 0..context.height() {
            solver.assert(
                context.bools().alloc(z3::ast::Int::distinct(
                    context.ctx(),
                    &(0..context.width())
                        .map(|col| context.get_cell(row, col))
                        .collect::<Vec<_>>(),
                )),
            );
        }
        for col in 0..context.width() {
            solver.assert(
                context.bools().alloc(z3::ast::Int::distinct(
                    context.ctx(),
                    &(0..context.height())
                        .map(|row| context.get_cell(row, col))
                        .collect::<Vec<_>>(),
                )),
            );
        }
    }
}

pub struct StandardBoxesConstraint;
impl Constraint for StandardBoxesConstraint {
    fn apply<'a>(&self, solver: &z3::Solver, context: &'a SudokuContext) {
        assert!(context.width() == SUDOKU_SIZE && context.height() == SUDOKU_SIZE);
        for x in 0..3 {
            for y in 0..3 {
                solver.assert(
                    context.bools().alloc(z3::ast::Int::distinct(
                        context.ctx(),
                        &(0..3)
                            .flat_map(|dx| {
                                (0..3).map(move |dy| context.get_cell(x * 3 + dx, y * 3 + dy))
                            })
                            .collect::<Vec<_>>(),
                    )),
                )
            }
        }
    }
}

pub struct GivenDigitConstraint {
    pub row: usize,
    pub col: usize,
    pub value: i32,
}
impl Constraint for GivenDigitConstraint {
    fn apply<'a>(&self, solver: &z3::Solver, context: &'a SudokuContext) {
        solver.assert(
            context.bools().alloc(
                context
                    .get_cell(self.row, self.col)
                    ._eq(context.const_int(self.value)),
            ),
        );
    }
}
