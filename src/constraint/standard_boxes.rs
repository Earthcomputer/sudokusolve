use z3::ast::Ast;
use crate::constraint::Constraint;
use crate::sudoku::{SUDOKU_SIZE, SudokuContext};

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
