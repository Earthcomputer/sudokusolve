use eframe::egui;
use z3::ast::Ast;
use macros::DynClone;
use crate::constraint::{ConfigurableConstraint, Constraint};
use crate::{sudoku, ui};
use crate::sudoku::SudokuContext;
use crate::ui::SudokuDrawContext;

#[derive(DynClone)]
#[dyn_clone(Constraint)]
pub struct KillerCageConstraint {
    cells: Vec<sudoku::Cell>,
    total: String,
}

impl Default for KillerCageConstraint {
    fn default() -> Self {
        Self {
            cells: Vec::new(),
            total: "0".to_owned(),
        }
    }
}

impl Constraint for KillerCageConstraint {
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
                .alloc(z3::ast::Int::distinct(context.ctx(), &cells)),
        );
        solver.assert(
            context
                .bools()
                .alloc(z3::ast::Int::add(context.ctx(), &cells)._eq(context.const_int(total))),
        );
    }
}

impl ConfigurableConstraint for KillerCageConstraint {
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
        "Killer Cage"
    }

    fn draw(&self, context: &SudokuDrawContext) {
        if let Some(top_left_cell) = self.cells.iter().min_by_key(|cell| (cell.row, cell.col)) {
            context.painter.text(
                context.cell_rect(top_left_cell.row, top_left_cell.col).min
                    + egui::Vec2::splat(4.0),
                egui::Align2::LEFT_TOP,
                &self.total,
                context.get_font(0.2),
                context.color,
            );
        }

        let cells: ahash::AHashSet<_> = self.cells.iter().copied().collect();
        for cell in &self.cells {
            let cell_rect = context.cell_rect(cell.row, cell.col);
            let cage_rect = cell_rect.shrink(ui::CELL_PADDING);

            let up = cell.row != 0 && cells.contains(&cell.up());
            let down = cells.contains(&cell.down());
            let left = cell.col != 0 && cells.contains(&cell.left());
            let right = cells.contains(&cell.right());

            let up_right = cell.row != 0 && cells.contains(&cell.up().right());
            let down_right = cells.contains(&cell.down().right());
            let up_left = cell.row != 0 && cell.col != 0 && cells.contains(&cell.up().left());
            let down_left = cell.col != 0 && cells.contains(&cell.down().left());

            if up {
                if right && !up_right {
                    Self::draw_dashed_line(
                        context.painter,
                        cage_rect.right_top(),
                        egui::Pos2::new(cell_rect.right(), cage_rect.top()),
                        context.color,
                    );
                }
                if left && !up_left {
                    Self::draw_dashed_line(
                        context.painter,
                        egui::Pos2::new(cell_rect.left(), cage_rect.top()),
                        cage_rect.left_top(),
                        context.color,
                    );
                }
            } else {
                Self::draw_dashed_line(
                    context.painter,
                    egui::Pos2::new(
                        if left {
                            cell_rect.left()
                        } else {
                            cage_rect.left()
                        },
                        cage_rect.top(),
                    ),
                    egui::Pos2::new(
                        if right {
                            cell_rect.right()
                        } else {
                            cage_rect.right()
                        },
                        cage_rect.top(),
                    ),
                    context.color,
                );
            }

            if down {
                if right && !down_right {
                    Self::draw_dashed_line(
                        context.painter,
                        cage_rect.right_bottom(),
                        egui::Pos2::new(cell_rect.right(), cage_rect.bottom()),
                        context.color,
                    );
                }
                if left && !down_left {
                    Self::draw_dashed_line(
                        context.painter,
                        egui::Pos2::new(cell_rect.left(), cage_rect.bottom()),
                        cage_rect.left_bottom(),
                        context.color,
                    );
                }
            } else {
                Self::draw_dashed_line(
                    context.painter,
                    egui::Pos2::new(
                        if left {
                            cell_rect.left()
                        } else {
                            cage_rect.left()
                        },
                        cage_rect.bottom(),
                    ),
                    egui::Pos2::new(
                        if right {
                            cell_rect.right()
                        } else {
                            cage_rect.right()
                        },
                        cage_rect.bottom(),
                    ),
                    context.color,
                );
            }

            if left {
                if up && !up_left {
                    Self::draw_dashed_line(
                        context.painter,
                        egui::Pos2::new(cage_rect.left(), cell_rect.top()),
                        cage_rect.left_top(),
                        context.color,
                    );
                }
                if down && !down_left {
                    Self::draw_dashed_line(
                        context.painter,
                        cage_rect.left_bottom(),
                        egui::Pos2::new(cage_rect.left(), cell_rect.bottom()),
                        context.color,
                    );
                }
            } else {
                Self::draw_dashed_line(
                    context.painter,
                    egui::Pos2::new(
                        cage_rect.left(),
                        if up { cell_rect.top() } else { cage_rect.top() },
                    ),
                    egui::Pos2::new(
                        cage_rect.left(),
                        if down {
                            cell_rect.bottom()
                        } else {
                            cage_rect.bottom()
                        },
                    ),
                    context.color,
                );
            }

            if right {
                if up && !up_right {
                    Self::draw_dashed_line(
                        context.painter,
                        egui::Pos2::new(cage_rect.right(), cell_rect.top()),
                        cage_rect.right_top(),
                        context.color,
                    );
                }
                if down && !down_right {
                    Self::draw_dashed_line(
                        context.painter,
                        cage_rect.right_bottom(),
                        egui::Pos2::new(cage_rect.right(), cell_rect.bottom()),
                        context.color,
                    );
                }
            } else {
                Self::draw_dashed_line(
                    context.painter,
                    egui::Pos2::new(
                        cage_rect.right(),
                        if up { cell_rect.top() } else { cage_rect.top() },
                    ),
                    egui::Pos2::new(
                        cage_rect.right(),
                        if down {
                            cell_rect.bottom()
                        } else {
                            cage_rect.bottom()
                        },
                    ),
                    context.color,
                );
            }
        }
    }

    fn draw_depth(&self) -> i32 {
        -5
    }
}

impl KillerCageConstraint {
    fn draw_dashed_line(
        painter: &egui::Painter,
        from: egui::Pos2,
        to: egui::Pos2,
        color: egui::Color32,
    ) {
        let total_len = from.distance(to);
        let dir = (to - from).normalized();

        let mut dist = 0.0;
        while dist < total_len {
            painter.line_segment(
                [from + dir * dist, from + dir * (dist + 2.0)],
                egui::Stroke::new(1.0, color),
            );
            dist += 4.0;
        }
    }
}
