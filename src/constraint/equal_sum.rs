use eframe::egui::{Context, Ui};
use z3::ast::Ast;
use crate::constraint::{ConfigurableConstraint, Constraint, renban};
use crate::sudoku;
use crate::sudoku::SudokuContext;
use macros::DynClone;
use z3::Solver;
use crate::ui::SudokuDrawContext;

#[derive(Default, DynClone)]
#[dyn_clone(Constraint + Send)]
pub struct EqualSumConstraint {
    cells: Vec<sudoku::Cell>,
}

impl Constraint for EqualSumConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        let mut cells_in_boxes = Vec::new();
        let mut current_box: Option<sudoku::Cell> = None;
        for &cell in &self.cells {
            let cell_box = sudoku::Cell::new(cell.row / 3, cell.col / 3);
            if !current_box.contains(&cell_box) {
                current_box = Some(cell_box);
                cells_in_boxes.push(Vec::new());
            }
            cells_in_boxes.last_mut().unwrap().push(cell);
        }
        if cells_in_boxes.len() < 2 {
            return;
        }
        let sums: Vec<_> = cells_in_boxes
            .iter()
            .map(|cells| {
                context.ints().alloc(z3::ast::Int::add(
                    context.ctx(),
                    &cells
                        .iter()
                        .map(|cell| context.get_cell(cell.row, cell.col))
                        .collect::<Vec<_>>(),
                ))
            })
            .collect();
        solver.assert(
            context.bools().alloc(z3::ast::Bool::and(
                context.ctx(),
                &sums
                    .array_windows::<2>()
                    .map(|[prev, next]| context.bools().alloc(prev._eq(next)))
                    .collect::<Vec<_>>(),
            )),
        )
    }
}

impl ConfigurableConstraint for EqualSumConstraint {
    fn configure(&mut self, _ctx: &Context, _ui: &mut Ui) {
    }

    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<sudoku::Cell>> {
        Some(&mut self.cells)
    }

    fn is_valid(&self) -> bool {
        !self.cells.is_empty()
    }

    fn name(&self) -> &'static str {
        "Equal Sum"
    }

    fn draw(&self, context: &SudokuDrawContext) {
        renban::draw_line_between_cells(&self.cells, context);
    }
}
