use crate::constraint::{ConfigurableConstraint, Constraint};
use crate::sudoku;
use crate::sudoku::SudokuContext;
use crate::ui::SudokuDrawContext;
use eframe::egui;
use eframe::egui::{Context, Ui};
use macros::DynClone;
use std::ops::Add;
use z3::ast::Ast;
use z3::Solver;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

pub fn iter_cells_in_dir(
    start_pos: sudoku::Cell,
    direction: Direction,
    context: &SudokuContext,
    mut f: impl FnMut(usize, sudoku::Cell),
) -> bool {
    let (dr, dc, iters) = match (direction, start_pos.row, start_pos.col) {
        (Direction::Horizontal, _, 0) => (0, 1, context.width()),
        (Direction::Vertical, 0, _) => (1, 0, context.height()),
        (Direction::Horizontal, _, col) => {
            if col != context.width() - 1 {
                return false;
            }
            (0, -1, context.width())
        }
        (Direction::Vertical, row, _) => {
            if row != context.height() - 1 {
                return false;
            }
            (-1, 0, context.height())
        }
    };

    for i in 0..iters {
        f(
            i,
            sudoku::Cell::new(
                (start_pos.row as isize + dr * i as isize) as usize,
                (start_pos.col as isize + dc * i as isize) as usize,
            ),
        );
    }

    true
}

pub fn draw_number_outside_grid(
    cells: &[sudoku::Cell],
    value: &str,
    direction: Direction,
    context: &SudokuDrawContext,
) {
    if cells.is_empty() {
        context.default_draw();
        return;
    }

    let pos = cells[0];
    let (dr, dc, align) = match (direction, pos.row, pos.col) {
        (Direction::Horizontal, _, 0) => (0, -1, egui::Align2::RIGHT_CENTER),
        (Direction::Vertical, 0, _) => (-1, 0, egui::Align2::CENTER_BOTTOM),
        (Direction::Horizontal, _, col) => {
            if col != context.width - 1 {
                context.default_draw();
                return;
            }
            (0, 1, egui::Align2::LEFT_CENTER)
        }
        (Direction::Vertical, row, _) => {
            if row != context.height - 1 {
                context.default_draw();
                return;
            }
            (1, 0, egui::Align2::CENTER_TOP)
        }
    };

    let cell_rect = context.cell_rect(pos.row, pos.col);
    context.painter.text(
        cell_rect
            .center()
            .add(egui::Vec2::new(dc as f32, dr as f32) * cell_rect.width() * 0.6),
        align,
        value,
        context.get_font(0.5),
        context.color,
    );
}

#[derive(DynClone)]
#[dyn_clone(Constraint)]
pub struct XSumConstraint {
    total: String,
    direction: Direction,
    cells: Vec<sudoku::Cell>,
}

impl Default for XSumConstraint {
    fn default() -> Self {
        Self {
            total: "0".to_owned(),
            direction: Direction::Horizontal,
            cells: Vec::new(),
        }
    }
}

impl Constraint for XSumConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        let start_pos = self.cells[0];
        let start_cell = context.get_cell(start_pos.row, start_pos.col);
        let mut sum = context.const_int(0);

        if !iter_cells_in_dir(start_pos, self.direction, context, |i, cell| {
            sum = context.ints().alloc(
                sum.add(
                    start_cell
                        .gt(context.const_int(i as i32))
                        .ite(context.get_cell(cell.row, cell.col), context.const_int(0)),
                ),
            );
        }) {
            return;
        }

        solver.assert(
            context
                .bools()
                .alloc(sum._eq(context.const_int(self.total.parse().unwrap()))),
        );
    }
}

impl ConfigurableConstraint for XSumConstraint {
    fn configure(&mut self, _ctx: &Context, ui: &mut Ui) {
        ui.label("Place the X-Sum on the perimeter of the grid with the appropriate direction.");
        ui.horizontal(|ui| {
            ui.label("Total");
            ui.text_edit_singleline(&mut self.total);
        });
        ui.horizontal(|ui| {
            ui.label("Direction");
            egui::ComboBox::from_id_source("choose_direction")
                .selected_text(match self.direction {
                    Direction::Horizontal => "Horizontal",
                    Direction::Vertical => "Vertical",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.direction, Direction::Horizontal, "Horizontal");
                    ui.selectable_value(&mut self.direction, Direction::Vertical, "Vertical");
                });
        });
    }

    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<sudoku::Cell>> {
        Some(&mut self.cells)
    }

    fn get_max_highlighted_cells(&self) -> usize {
        1
    }

    fn is_valid(&self) -> bool {
        !self.cells.is_empty() && self.total.parse::<i32>().is_ok()
    }

    fn name(&self) -> &'static str {
        "X-Sum"
    }

    fn draw(&self, context: &SudokuDrawContext) {
        draw_number_outside_grid(&self.cells, &self.total, self.direction, context);
    }
}
