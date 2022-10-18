use eframe::egui::{Context, Ui};
use crate::constraint::{ConfigurableConstraint, Constraint, renban};
use crate::sudoku;
use crate::sudoku::SudokuContext;
use macros::DynClone;
use z3::Solver;
use crate::ui::SudokuDrawContext;

#[derive(Default, DynClone)]
#[dyn_clone(Constraint + Send)]
pub struct GermanWhisperConstraint {
    cells: Vec<sudoku::Cell>,
}

impl Constraint for GermanWhisperConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        for [prev, next] in self.cells.array_windows::<2>() {
            let delta = z3::ast::Int::sub(
                context.ctx(),
                &[
                    context.get_cell(next.row, next.col),
                    context.get_cell(prev.row, prev.col),
                ],
            );
            let minus_delta = delta.unary_minus();
            solver.assert(context.bools().alloc(z3::ast::Bool::or(
                context.ctx(),
                &[
                    context.bools().alloc(delta.ge(context.const_int(5))),
                    context.bools().alloc(minus_delta.ge(context.const_int(5))),
                ],
            )));
        }
    }
}

impl ConfigurableConstraint for GermanWhisperConstraint {
    fn configure(&mut self, _ctx: &Context, _ui: &mut Ui) {
    }

    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<sudoku::Cell>> {
        Some(&mut self.cells)
    }

    fn is_valid(&self) -> bool {
        self.cells.len() >= 2
    }

    fn name(&self) -> &'static str {
        "German Whisper"
    }

    fn draw(&self, context: &SudokuDrawContext) {
        renban::draw_line_between_cells(&self.cells, context);
    }
}
