use z3::ast::Ast;
use crate::constraint::Constraint;
use crate::sudoku::SudokuContext;

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
