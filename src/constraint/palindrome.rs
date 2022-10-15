use eframe::egui::{Context, Ui};
use crate::constraint::{ConfigurableConstraint, Constraint, renban};
use crate::sudoku;
use crate::sudoku::SudokuContext;
use macros::DynClone;
use z3::ast::Ast;
use z3::Solver;
use crate::ui::SudokuDrawContext;

#[derive(Default, DynClone)]
#[dyn_clone(Constraint)]
pub struct PalindromeConstraint {
    cells: Vec<sudoku::Cell>,
}

impl Constraint for PalindromeConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        for i in 0..self.cells.len() / 2 {
            solver.assert(
                context
                    .bools()
                    .alloc(context.get_cell(self.cells[i].row, self.cells[i].col)._eq(
                        context.get_cell(
                            self.cells[self.cells.len() - 1 - i].row,
                            self.cells[self.cells.len() - 1 - i].col,
                        ),
                    )),
            );
        }
    }
}

impl ConfigurableConstraint for PalindromeConstraint {
    fn configure(&mut self, _ctx: &Context, _ui: &mut Ui) {
    }

    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<sudoku::Cell>> {
        Some(&mut self.cells)
    }

    fn is_valid(&self) -> bool {
        self.cells.len() >= 2
    }

    fn name(&self) -> &'static str {
        "Palindrome"
    }

    fn draw(&self, context: &SudokuDrawContext) {
        renban::draw_line_between_cells(&self.cells, context);
    }
}
