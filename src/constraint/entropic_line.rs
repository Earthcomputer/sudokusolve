use std::ops::Sub;
use crate::constraint::{renban, ConfigurableConstraint, Constraint};
use crate::sudoku;
use crate::sudoku::SudokuContext;
use crate::ui::SudokuDrawContext;
use eframe::egui::{Context, Ui};
use macros::DynClone;
use z3::ast::Ast;
use z3::Solver;

#[derive(Default, DynClone)]
#[dyn_clone(Constraint + Send)]
pub struct EntropicLineConstraint {
    cells: Vec<sudoku::Cell>,
}

impl Constraint for EntropicLineConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        let divisor = context.const_int(
            ((*context.digits_range().end() - *context.digits_range().start() + 2) / 3) as i32,
        );
        let offset = context.const_int((*context.digits_range().start() % 3) as i32);

        for window in self.cells.windows(3) {
            solver.assert(
                context.bools().alloc(z3::ast::Int::distinct(
                    context.ctx(),
                    &window
                        .iter()
                        .map(|cell| {
                            context
                                .ints()
                                .alloc(context.get_cell(cell.row, cell.col).sub(offset).div(divisor))
                        })
                        .collect::<Vec<_>>(),
                )),
            )
        }
    }
}

impl ConfigurableConstraint for EntropicLineConstraint {
    fn configure(&mut self, _ctx: &Context, _ui: &mut Ui) {}

    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<sudoku::Cell>> {
        Some(&mut self.cells)
    }

    fn is_valid(&self) -> bool {
        self.cells.len() >= 3
    }

    fn name(&self) -> &'static str {
        "Entropic Line"
    }

    fn draw(&self, context: &SudokuDrawContext) {
        renban::draw_line_between_cells(&self.cells, context);
    }
}
