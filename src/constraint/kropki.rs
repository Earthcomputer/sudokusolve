use crate::constraint::{ConfigurableConstraint, Constraint};
use crate::sudoku;
use crate::sudoku::SudokuContext;
use crate::ui::SudokuDrawContext;
use ahash::AHashSet;
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
    draw_dot(rect_a.union(rect_b).center(), rect_a.width() * 0.1);
}

fn white_kropki_constraint<'a>(
    a: sudoku::Cell,
    b: sudoku::Cell,
    context: &'a SudokuContext<'a>,
) -> z3::ast::Bool<'a> {
    let a = context.get_cell(a.row, a.col);
    let b = context.get_cell(b.row, b.col);
    z3::ast::Bool::or(
        context.ctx(),
        &[
            context
                .bools()
                .alloc(a._eq(context.ints().alloc(b.add(context.const_int(1))))),
            context
                .bools()
                .alloc(b._eq(context.ints().alloc(a.add(context.const_int(1))))),
        ],
    )
}

#[derive(Default, DynClone)]
#[dyn_clone(Constraint + Send)]
pub struct WhiteKropkiConstraint {
    cells: Vec<sudoku::Cell>,
}

impl Constraint for WhiteKropkiConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        solver.assert(context.bools().alloc(white_kropki_constraint(
            self.cells[0],
            self.cells[1],
            context,
        )));
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

fn black_kropki_constraint<'a>(
    a: sudoku::Cell,
    b: sudoku::Cell,
    context: &'a SudokuContext<'a>,
) -> z3::ast::Bool<'a> {
    let a = context.get_cell(a.row, a.col);
    let b = context.get_cell(b.row, b.col);
    z3::ast::Bool::or(
        context.ctx(),
        &[
            context
                .bools()
                .alloc(a._eq(context.ints().alloc(b.mul(context.const_int(2))))),
            context
                .bools()
                .alloc(b._eq(context.ints().alloc(a.mul(context.const_int(2))))),
        ],
    )
}

#[derive(Default, DynClone)]
#[dyn_clone(Constraint + Send)]
pub struct BlackKropkiConstraint {
    cells: Vec<sudoku::Cell>,
}

impl Constraint for BlackKropkiConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        solver.assert(context.bools().alloc(black_kropki_constraint(
            self.cells[0],
            self.cells[1],
            context,
        )));
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

fn find_kropki_dots(context: &SudokuContext) -> AHashSet<(sudoku::Cell, sudoku::Cell)> {
    context
        .constraints()
        .iter()
        .filter_map(|constraint| {
            let cells = if let Some(white) = constraint.downcast::<WhiteKropkiConstraint>() {
                &white.cells
            } else if let Some(black) = constraint.downcast::<BlackKropkiConstraint>() {
                &black.cells
            } else {
                return None;
            };
            let first = cells[0];
            let second = cells[1];
            match (first.row == second.row, first.col == second.col) {
                (true, true) => {
                    unreachable!("The ui should prevent two of the same cell in a constraint")
                }
                (true, false) => Some((
                    sudoku::Cell::new(first.row, first.col.min(second.col)),
                    sudoku::Cell::new(second.row, first.col.max(second.col)),
                )),
                (false, true) => Some((
                    sudoku::Cell::new(first.row.min(second.row), first.col),
                    sudoku::Cell::new(first.row.max(second.row), second.col),
                )),
                (false, false) => None,
            }
        })
        .collect()
}

fn negative_constraint(
    solver: &Solver,
    context: &SudokuContext,
    constraint: impl for<'a> Fn(sudoku::Cell, sudoku::Cell, &'a SudokuContext<'a>) -> z3::ast::Bool<'a>,
) {
    let dots = find_kropki_dots(context);
    for row in 0..context.height() {
        for col in 0..context.width() - 1 {
            let a = sudoku::Cell::new(row, col);
            let b = sudoku::Cell::new(row, col + 1);
            if !dots.contains(&(a, b)) {
                solver.assert(context.bools().alloc(constraint(a, b, context).not()));
            }
        }
    }
    for row in 0..context.height() - 1 {
        for col in 0..context.width() {
            let a = sudoku::Cell::new(row, col);
            let b = sudoku::Cell::new(row + 1, col);
            if !dots.contains(&(a, b)) {
                solver.assert(context.bools().alloc(constraint(a, b, context).not()));
            }
        }
    }
}

#[derive(Default, DynClone)]
#[dyn_clone(Constraint + Send)]
pub struct NegativeWhiteKropkiConstraint;

impl Constraint for NegativeWhiteKropkiConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        negative_constraint(solver, context, white_kropki_constraint);
    }
}

impl ConfigurableConstraint for NegativeWhiteKropkiConstraint {
    fn configure(&mut self, _ctx: &Context, _ui: &mut Ui) {}

    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<sudoku::Cell>> {
        None
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "White Kropki Dots (Negative Constraint)"
    }
}

#[derive(Default, DynClone)]
#[dyn_clone(Constraint + Send)]
pub struct NegativeBlackKropkiConstraint;

impl Constraint for NegativeBlackKropkiConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        negative_constraint(solver, context, black_kropki_constraint);
    }
}

impl ConfigurableConstraint for NegativeBlackKropkiConstraint {
    fn configure(&mut self, _ctx: &Context, _ui: &mut Ui) {}

    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<sudoku::Cell>> {
        None
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "Black Kropki Dots (Negative Constraint)"
    }
}
