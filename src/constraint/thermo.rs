use crate::constraint::{ConfigurableConstraint, Constraint};
use crate::sudoku;
use crate::sudoku::SudokuContext;
use crate::ui::SudokuDrawContext;
use eframe::egui;
use eframe::egui::{Context, Ui};
use macros::DynClone;
use z3::Solver;

#[derive(Default, DynClone)]
#[dyn_clone(Constraint + Send)]
pub struct ThermoConstraint {
    cells: Vec<sudoku::Cell>,
}

impl Constraint for ThermoConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        for &[prev, next] in self.cells.array_windows::<2>() {
            solver.assert(
                context.bools().alloc(
                    context
                        .get_cell(prev.row, prev.col)
                        .lt(context.get_cell(next.row, next.col)),
                ),
            );
        }
    }
}

impl ConfigurableConstraint for ThermoConstraint {
    fn configure(&mut self, _ctx: &Context, _ui: &mut Ui) {}

    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<sudoku::Cell>> {
        Some(&mut self.cells)
    }

    fn is_valid(&self) -> bool {
        !self.cells.is_empty()
    }

    fn name(&self) -> &'static str {
        "Thermo"
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

        let bulb_radius = 0.35;
        let column_radius = 0.15;

        let first_rect = context.cell_rect(self.cells[0].row, self.cells[0].col);
        context
            .painter
            .circle_filled(first_rect.center(), first_rect.width() * bulb_radius, context.color);

        for [prev, next] in self.cells.array_windows::<2>() {
            let prev_pos = context.cell_rect(prev.row, prev.col);
            let next_pos = context.cell_rect(next.row, next.col);
            let vec = next_pos.center() - prev_pos.center();
            let side = vec.rot90().normalized() * prev_pos.width() * column_radius;
            context.painter.add(egui::epaint::PathShape::convex_polygon(
                vec![
                    next_pos.center() - side,
                    next_pos.center() + side,
                    prev_pos.center() + side,
                    prev_pos.center() - side,
                ],
                context.color,
                egui::Stroke::none(),
            ));
            context
                .painter
                .circle_filled(next_pos.center(), next_pos.width() * column_radius, context.color);
        }
    }
}
