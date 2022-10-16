use crate::constraint::{ConfigurableConstraint, Constraint};
use crate::sudoku;
use crate::sudoku::SudokuContext;
use crate::ui::SudokuDrawContext;
use eframe::egui;
use eframe::egui::{Context, Ui};
use macros::DynClone;
use z3::ast::Ast;
use z3::Solver;

#[derive(Default, DynClone)]
#[dyn_clone(Constraint)]
pub struct ArrowConstraint {
    cells: Vec<sudoku::Cell>,
}

impl Constraint for ArrowConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        solver.assert(
            context.bools().alloc(
                z3::ast::Int::add(
                    context.ctx(),
                    &self.cells[1..]
                        .iter()
                        .map(|cell| context.get_cell(cell.row, cell.col))
                        .collect::<Vec<_>>(),
                )
                ._eq(context.get_cell(self.cells[0].row, self.cells[0].col)),
            ),
        );
    }
}

impl ConfigurableConstraint for ArrowConstraint {
    fn configure(&mut self, _ctx: &Context, _ui: &mut Ui) {}

    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<sudoku::Cell>> {
        Some(&mut self.cells)
    }

    fn is_valid(&self) -> bool {
        self.cells.len() >= 2
    }

    fn name(&self) -> &'static str {
        "Arrow"
    }

    fn draw(&self, context: &SudokuDrawContext) {
        if self.cells.is_empty() {
            context.default_draw();
            return;
        }

        for [prev, next] in self.cells.array_windows::<2>() {
            if prev.row.abs_diff(next.row) > 1 || prev.col.abs_diff(next.col) > 1 {
                context.default_draw();
                return;
            }
        }

        let stroke = egui::Stroke::new(1f32, context.color);

        let circle_radius = 0.4;
        let first_rect = context.cell_rect(self.cells[0].row, self.cells[0].col);
        context.painter.circle_stroke(
            first_rect.center(),
            first_rect.width() * circle_radius,
            stroke,
        );

        for (index, [prev, next]) in self.cells.array_windows::<2>().enumerate() {
            let prev_center = context.cell_rect(prev.row, prev.col).center();
            let next_center = context.cell_rect(next.row, next.col).center();
            context.painter.line_segment(
                [
                    if index == 0 {
                        prev_center
                            + (next_center - prev_center).normalized()
                                * first_rect.width()
                                * circle_radius
                    } else {
                        prev_center
                    },
                    next_center,
                ],
                stroke,
            );

            if index == self.cells.len() - 2 {
                // draw the arrow tip
                let arrow_tip_length = 0.2;
                let rot = egui::emath::Rot2::from_angle(std::f32::consts::PI * 3.0 / 4.0);
                let vec = (next_center - prev_center).normalized()
                    * first_rect.width()
                    * arrow_tip_length;
                context
                    .painter
                    .line_segment([next_center, next_center + rot * vec], stroke);
                context
                    .painter
                    .line_segment([next_center, next_center + rot.inverse() * vec], stroke);
            }
        }
    }
}
