use eframe::egui;
use z3::ast::Ast;
use macros::DynClone;
use crate::constraint::{ConfigurableConstraint, Constraint};
use crate::sudoku;
use crate::sudoku::SudokuContext;
use crate::ui::SudokuDrawContext;

#[derive(DynClone)]
#[dyn_clone(Constraint)]
pub struct LittleKillerConstraint {
    cells: Vec<sudoku::Cell>,
    total: String,
}

impl Default for LittleKillerConstraint {
    fn default() -> Self {
        Self {
            cells: Vec::new(),
            total: "0".to_owned(),
        }
    }
}

impl Constraint for LittleKillerConstraint {
    fn apply<'a>(&self, solver: &z3::Solver, context: &'a SudokuContext) {
        let total: i32 = self.total.parse().unwrap();

        let cells = self
            .cells
            .iter()
            .map(|cell| context.get_cell(cell.row, cell.col))
            .collect::<Vec<_>>();
        solver.assert(
            context
                .bools()
                .alloc(z3::ast::Int::add(context.ctx(), &cells)._eq(context.const_int(total))),
        );
    }
}

impl ConfigurableConstraint for LittleKillerConstraint {
    fn configure(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Total");
            ui.text_edit_singleline(&mut self.total);
        });
    }

    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<sudoku::Cell>> {
        Some(&mut self.cells)
    }

    fn is_valid(&self) -> bool {
        !self.cells.is_empty() && self.total.parse::<i32>().is_ok()
    }

    fn name(&self) -> &'static str {
        return "Little Killer";
    }

    fn draw(&self, context: &SudokuDrawContext) {
        let mut top_cell: Option<sudoku::Cell> = None;
        let mut positive_gradient: Option<bool> = None;
        let mut valid = true;
        for cell in &self.cells {
            if let Some(top) = top_cell {
                if top.row.abs_diff(cell.row) != top.col.abs_diff(cell.col) {
                    valid = false;
                    break;
                }

                if let Some(positive_gradient) = positive_gradient {
                    if ((cell.row > top.row) == (cell.col > top.col)) != positive_gradient {
                        valid = false;
                        break;
                    }
                } else {
                    positive_gradient = Some((cell.row > top.row) == (cell.col > top.col));
                }

                if cell.row < top.row {
                    top_cell = Some(*cell);
                }
            } else {
                top_cell = Some(*cell);
            }
        }

        if !valid || top_cell.is_none() {
            context.default_draw();
            return;
        }

        let top_cell = top_cell.unwrap();

        // check that the top cell is on the perimeter of the grid, and is coming away from it
        if top_cell.row != 0
            && (positive_gradient == Some(false) || top_cell.col != 0)
            && (positive_gradient == Some(true) || top_cell.col != context.width - 1)
        {
            context.default_draw();
            return;
        }

        let bottom_cell = match positive_gradient {
            Some(true) => {
                let right_cell = sudoku::Cell::new(
                    top_cell.row + context.width - 1 - top_cell.col,
                    context.width - 1,
                );
                if right_cell.row >= context.height {
                    sudoku::Cell::new(
                        context.height - 1,
                        top_cell.col + context.height - 1 - top_cell.row,
                    )
                } else {
                    right_cell
                }
            }
            Some(false) => {
                let left_cell = sudoku::Cell::new(top_cell.row + top_cell.col, 0);
                if left_cell.row >= context.height {
                    sudoku::Cell::new(
                        context.height - 1,
                        top_cell.col - (context.height - 1 - top_cell.row),
                    )
                } else {
                    left_cell
                }
            }
            None => {
                // check that the top cell is on the bottom perimeter of the grid
                if top_cell.row != context.height - 1
                    && (positive_gradient == Some(true) || top_cell.col != 0)
                    && (positive_gradient == Some(false) || top_cell.col != context.width - 1)
                {
                    context.default_draw();
                    return;
                }
                top_cell
            }
        };

        // check that we have filled the whole diagonal between the top and bottom
        if self.cells.len() != bottom_cell.row - top_cell.row + 1 {
            context.default_draw();
            return;
        }

        let positive_gradient =
            positive_gradient.unwrap_or_else(|| (top_cell.row == 0) ^ (top_cell.col == 0));
        let mut arrow_tip = if positive_gradient {
            context.cell_rect(top_cell.row, top_cell.col).min
        } else {
            context.cell_rect(top_cell.row, top_cell.col).right_top()
        };
        let adjust = 2.0;
        arrow_tip.y -= adjust;
        arrow_tip.x += if positive_gradient { -adjust } else { adjust };
        let arrow_size = 12f32;
        let arrow_tail = arrow_tip
            + egui::Vec2::new(
            if positive_gradient {
                -arrow_size
            } else {
                arrow_size
            },
            -arrow_size,
        );
        Self::draw_arrow(context.painter, arrow_tail, arrow_tip, context.color);

        context.painter.text(
            arrow_tail,
            if positive_gradient {
                egui::Align2::RIGHT_BOTTOM
            } else {
                egui::Align2::LEFT_BOTTOM
            },
            &self.total,
            egui::TextStyle::Body.resolve(context.style),
            context.color,
        );
    }
}

impl LittleKillerConstraint {
    fn draw_arrow(
        painter: &egui::Painter,
        tail: egui::Pos2,
        tip: egui::Pos2,
        color: egui::Color32,
    ) {
        let arrow_thickness = 2f32;

        let stroke = egui::Stroke::new(arrow_thickness, color);
        painter.line_segment([tail, tip], stroke);

        let rot = egui::emath::Rot2::from_angle(std::f32::consts::PI * 3.0 / 4.0);
        let vec = tip - tail;
        painter.line_segment([tip, tip + rot * vec], stroke);
        painter.line_segment([tip, tip + rot.inverse() * vec], stroke);
    }
}
