use z3::ast::Ast;
use crate::constraint::Constraint;
use crate::sudoku::SudokuContext;

pub struct DigitDefinitionConstraint;

impl Constraint for DigitDefinitionConstraint {
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
