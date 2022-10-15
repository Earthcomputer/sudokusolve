use crate::sudoku::{SudokuContext, SUDOKU_SIZE};
use crate::ui::SudokuDrawContext;
use crate::{sudoku, ui, DynClone};
use eframe::egui;
use macros::DynClone;
use z3::ast::Ast;

pub trait Constraint {
    fn apply<'a>(&self, solver: &z3::Solver, context: &'a SudokuContext);
}

pub struct DigitsConstraint;
impl Constraint for DigitsConstraint {
    fn apply<'a>(&self, solver: &z3::Solver, context: &'a SudokuContext) {
        for cell in context.all_cells() {
            solver.assert(
                context.bools().alloc(z3::ast::Bool::or(
                    context.ctx(),
                    &(1..=9)
                        .map(|i| context.bools().alloc(cell._eq(context.const_int(i))))
                        .collect::<Vec<_>>(),
                )),
            );
        }
    }
}

pub struct LatinSquareConstraint;
impl Constraint for LatinSquareConstraint {
    fn apply<'a>(&self, solver: &z3::Solver, context: &'a SudokuContext) {
        for row in 0..context.height() {
            solver.assert(
                context.bools().alloc(z3::ast::Int::distinct(
                    context.ctx(),
                    &(0..context.width())
                        .map(|col| context.get_cell(row, col))
                        .collect::<Vec<_>>(),
                )),
            );
        }
        for col in 0..context.width() {
            solver.assert(
                context.bools().alloc(z3::ast::Int::distinct(
                    context.ctx(),
                    &(0..context.height())
                        .map(|row| context.get_cell(row, col))
                        .collect::<Vec<_>>(),
                )),
            );
        }
    }
}

pub struct StandardBoxesConstraint;
impl Constraint for StandardBoxesConstraint {
    fn apply<'a>(&self, solver: &z3::Solver, context: &'a SudokuContext) {
        assert!(context.width() == SUDOKU_SIZE && context.height() == SUDOKU_SIZE);
        for x in 0..3 {
            for y in 0..3 {
                solver.assert(
                    context.bools().alloc(z3::ast::Int::distinct(
                        context.ctx(),
                        &(0..3)
                            .flat_map(|dx| {
                                (0..3).map(move |dy| context.get_cell(x * 3 + dx, y * 3 + dy))
                            })
                            .collect::<Vec<_>>(),
                    )),
                )
            }
        }
    }
}

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

pub trait ConfigurableConstraint: Constraint + DynClone<dyn Constraint> {
    fn configure(&mut self, ctx: &egui::Context, ui: &mut egui::Ui);
    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<sudoku::Cell>>;
    fn get_max_highlighted_cells(&self) -> usize {
        usize::MAX
    }
    fn is_valid(&self) -> bool;
    fn name(&self) -> &'static str;
    fn draw(&self, context: &SudokuDrawContext) {
        context.default_draw();
    }
}

pub static CONFIGURABLES: phf::Map<&'static str, fn() -> Box<dyn ConfigurableConstraint>> = phf::phf_map! {
    "Killer Cage" => || Box::new(KillerCageConstraint::default()),
    "Little Killer" => || Box::new(LittleKillerConstraint::default()),
};

pub fn make_default_constraint() -> Box<dyn ConfigurableConstraint> {
    Box::new(KillerCageConstraint::default())
}

#[derive(DynClone)]
#[dyn_clone(Constraint)]
struct KillerCageConstraint {
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
        return "Killer Cage";
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

#[derive(DynClone)]
#[dyn_clone(Constraint)]
struct LittleKillerConstraint {
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
