use crate::constraint::{ConfigurableConstraint, Constraint};
use crate::sudoku;
use crate::sudoku::SudokuContext;
use crate::ui::SudokuDrawContext;
use eframe::egui;
use eframe::egui::{Context, Ui};
use macros::DynClone;
use z3::ast::Ast;
use z3::Solver;

pub fn draw_line_between_cells(cells: &[sudoku::Cell], context: &SudokuDrawContext) {
    for [prev, next] in cells.array_windows::<2>() {
        if prev.row.abs_diff(next.row) > 1 || prev.col.abs_diff(next.col) > 1 {
            context.default_draw();
            return;
        }
    }

    let renban_width = 0.1;

    for [prev, next] in cells.array_windows::<2>() {
        let prev_rect = context.cell_rect(prev.row, prev.col);
        let next_rect = context.cell_rect(next.row, next.col);
        context.painter.line_segment(
            [prev_rect.center(), next_rect.center()],
            egui::Stroke::new(prev_rect.width() * renban_width, context.color),
        );
    }

    for cell in cells {
        let rect = context.cell_rect(cell.row, cell.col);
        context
            .painter
            .circle_filled(rect.center(), rect.width() * renban_width * 0.5, context.color);
    }
}

#[derive(Default, DynClone)]
#[dyn_clone(Constraint + Send)]
pub struct RenbanConstraint {
    cells: Vec<sudoku::Cell>,
}

impl Constraint for RenbanConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        // The renban constraint can be subdivided into two separate constraints:
        // 1. All digits on the renban are distinct...
        solver.assert(
            context.bools().alloc(z3::ast::Int::distinct(
                context.ctx(),
                &self
                    .cells
                    .iter()
                    .map(|cell| context.get_cell(cell.row, cell.col))
                    .collect::<Vec<_>>(),
            )),
        );
        // 2. All digits on the renban are consecutive. If all digits are distinct, then
        // consecutiveness implies that the minimum digit is n-1 different from the maximum digit,
        // or suffice to say that no pair of digits on the renban differ by n or more.
        // More formally, we say that there does *not* exist a pair of integers (x, y) such that
        // x is on the renban, y is on the renban, and y - x >= n.

        let mut renban_set = z3::ast::Set::empty(context.ctx(), context.int_type());
        for cell in &self.cells {
            renban_set = renban_set.add(context.get_cell(cell.row, cell.col));
        }
        let renban_set = context.sets().alloc(renban_set);

        let x = context
            .ints()
            .alloc(z3::ast::Int::fresh_const(context.ctx(), "x"));
        let y = context
            .ints()
            .alloc(z3::ast::Int::fresh_const(context.ctx(), "y"));
        let x_is_on_renban = context.bools().alloc(z3::ast::Set::member(renban_set, x));
        let y_is_on_renban = context.bools().alloc(z3::ast::Set::member(renban_set, y));

        solver.assert(
            context.bools().alloc(
                z3::ast::exists_const(
                    context.ctx(),
                    &[x, y],
                    &[context.patterns().alloc(z3::Pattern::new(
                        context.ctx(),
                        &[x_is_on_renban, y_is_on_renban],
                    ))],
                    context.bools().alloc(z3::ast::Bool::and(
                        context.ctx(),
                        &[
                            x_is_on_renban,
                            y_is_on_renban,
                            context.bools().alloc(z3::ast::Bool::or(
                                context.ctx(),
                                &[context.bools().alloc(
                                    z3::ast::Int::sub(context.ctx(), &[y, x])
                                        .ge(context.const_int(self.cells.len() as i32)),
                                )],
                            )),
                        ],
                    )),
                )
                .not(),
            ),
        )
    }
}

impl ConfigurableConstraint for RenbanConstraint {
    fn configure(&mut self, _ctx: &Context, _ui: &mut Ui) {}

    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<sudoku::Cell>> {
        Some(&mut self.cells)
    }

    fn is_valid(&self) -> bool {
        self.cells.len() >= 2
    }

    fn name(&self) -> &'static str {
        "Renban"
    }

    fn draw(&self, context: &SudokuDrawContext) {
        draw_line_between_cells(&self.cells, context);
    }
}
