use crate::constraint::{ConfigurableConstraint, Constraint};
use crate::sudoku;
use crate::sudoku::SudokuContext;
use crate::ui::SudokuDrawContext;
use eframe::egui;
use eframe::egui::{Context, Ui};
use macros::DynClone;
use std::ops::{Add, Mul};
use z3::ast::Ast;
use z3::Solver;

fn draw_kropki_dot(
    cells: &[sudoku::Cell],
    context: &SudokuDrawContext,
    draw_dot: impl FnOnce(egui::Pos2, f32),
) {
    if cells.len() != 2 {
        context.default_draw();
        return;
    }

    let dr = cells[0].row.abs_diff(cells[1].row);
    let dc = cells[0].col.abs_diff(cells[1].col);
    let valid = (dr == 1 && dc == 0) || (dr == 0 && dc == 1);
    if !valid {
        context.default_draw();
        return;
    }

    let rect_a = context.cell_rect(cells[0].row, cells[0].col);
    let rect_b = context.cell_rect(cells[1].row, cells[1].col);
    draw_dot(
        rect_a.union(rect_b).center(),
        rect_a.width() * 0.1,
    );
}

#[derive(Default, DynClone)]
#[dyn_clone(Constraint)]
pub struct WhiteKropkiConstraint {
    cells: Vec<sudoku::Cell>,
}

impl Constraint for WhiteKropkiConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        let a = context.get_cell(self.cells[0].row, self.cells[0].col);
        let b = context.get_cell(self.cells[1].row, self.cells[1].col);
        solver.assert(
            context.bools().alloc(z3::ast::Bool::or(
                context.ctx(),
                &[
                    context
                        .bools()
                        .alloc(a._eq(context.ints().alloc(b.add(context.const_int(1))))),
                    context
                        .bools()
                        .alloc(b._eq(context.ints().alloc(a.add(context.const_int(1))))),
                ],
            )),
        );
    }
}

impl ConfigurableConstraint for WhiteKropkiConstraint {
    fn configure(&mut self, _ctx: &Context, _ui: &mut Ui) {}

    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<sudoku::Cell>> {
        Some(&mut self.cells)
    }

    fn get_max_highlighted_cells(&self) -> usize {
        2
    }

    fn is_valid(&self) -> bool {
        self.cells.len() == 2
    }

    fn name(&self) -> &'static str {
        "White Kropki Dot"
    }

    fn draw(&self, context: &SudokuDrawContext) {
        draw_kropki_dot(&self.cells, context, |center, radius| {
            context
                .painter
                .circle_stroke(center, radius, egui::Stroke::new(1f32, context.color));
        });
    }
}

#[derive(Default, DynClone)]
#[dyn_clone(Constraint)]
pub struct BlackKropkiConstraint {
    cells: Vec<sudoku::Cell>,
}

impl Constraint for BlackKropkiConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        let a = context.get_cell(self.cells[0].row, self.cells[0].col);
        let b = context.get_cell(self.cells[1].row, self.cells[1].col);
        solver.assert(
            context.bools().alloc(z3::ast::Bool::or(
                context.ctx(),
                &[
                    context
                        .bools()
                        .alloc(a._eq(context.ints().alloc(b.mul(context.const_int(2))))),
                    context
                        .bools()
                        .alloc(b._eq(context.ints().alloc(a.mul(context.const_int(2))))),
                ],
            )),
        );
    }
}

impl ConfigurableConstraint for BlackKropkiConstraint {
    fn configure(&mut self, _ctx: &Context, _ui: &mut Ui) {}

    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<sudoku::Cell>> {
        Some(&mut self.cells)
    }

    fn get_max_highlighted_cells(&self) -> usize {
        2
    }

    fn is_valid(&self) -> bool {
        self.cells.len() == 2
    }

    fn name(&self) -> &'static str {
        "Black Kropki Dot"
    }

    fn draw(&self, context: &SudokuDrawContext) {
        draw_kropki_dot(&self.cells, context, |center, radius| {
            context.painter.circle_filled(center, radius, context.color);
        });
    }
}
