use crate::constraint::{ConfigurableConstraint, Constraint};
use crate::sudoku::{Cell, SudokuContext};
use crate::ui::SudokuDrawContext;
use eframe::egui;
use eframe::egui::{Context, Ui};
use macros::DynClone;
use z3::ast::Ast;
use z3::Solver;

#[derive(Clone, Eq, PartialEq)]
enum Direction {
    Positive,
    Negative,
    Both,
}

#[derive(DynClone)]
#[dyn_clone(Constraint)]
pub struct DiagonalConstraint {
    direction: Direction,
}

impl Default for DiagonalConstraint {
    fn default() -> Self {
        Self {
            direction: Direction::Both,
        }
    }
}

impl Constraint for DiagonalConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        if context.width() != context.height() {
            return;
        }

        if self.direction != Direction::Positive {
            solver.assert(
                context.bools().alloc(z3::ast::Int::distinct(
                    context.ctx(),
                    &(0..context.width())
                        .map(|pos| context.get_cell(pos, pos))
                        .collect::<Vec<_>>(),
                )),
            );
        }
        if self.direction != Direction::Negative {
            solver.assert(
                context.bools().alloc(z3::ast::Int::distinct(
                    context.ctx(),
                    &(0..context.width())
                        .map(|pos| context.get_cell(pos, context.width() - 1 - pos))
                        .collect::<Vec<_>>(),
                )),
            );
        }
    }
}

impl ConfigurableConstraint for DiagonalConstraint {
    fn configure(&mut self, _ctx: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Direction");
            egui::ComboBox::from_id_source("diagonal_combo")
                .selected_text(match self.direction {
                    Direction::Positive => "Positive",
                    Direction::Negative => "Negative",
                    Direction::Both => "Both",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.direction, Direction::Positive, "Positive");
                    ui.selectable_value(&mut self.direction, Direction::Negative, "Negative");
                    ui.selectable_value(&mut self.direction, Direction::Both, "Both");
                });
        });
    }

    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<Cell>> {
        None
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "Diagonal"
    }

    fn draw(&self, context: &SudokuDrawContext) {
        if context.width != context.height {
            context.default_draw();
            return;
        }

        if self.direction != Direction::Positive {
            context.painter.line_segment(
                [
                    context.cell_rect(0, 0).left_top(),
                    context
                        .cell_rect(context.height - 1, context.width - 1)
                        .right_bottom(),
                ],
                egui::Stroke::new(1f32, context.color),
            );
        }
        if self.direction != Direction::Negative {
            context.painter.line_segment(
                [
                    context.cell_rect(0, context.width - 1).right_top(),
                    context.cell_rect(context.height - 1, 0).left_bottom(),
                ],
                egui::Stroke::new(1f32, context.color),
            );
        }
    }

    fn always_draw(&self) -> bool {
        true
    }

    fn draw_depth(&self) -> i32 {
        -10
    }
}
