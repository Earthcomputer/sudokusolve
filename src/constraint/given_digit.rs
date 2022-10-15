use z3::ast::Ast;
use crate::constraint::Constraint;
use crate::sudoku::SudokuContext;

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
