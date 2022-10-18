use eframe::egui::{Context, Ui};
use crate::constraint::{ConfigurableConstraint, Constraint};
use crate::sudoku::{Cell, SudokuContext};
use macros::DynClone;
use z3::ast::Ast;
use z3::Solver;

const KNIGHT_DELTAS: [(isize, isize); 4] = [(1, 2), (2, 1), (-1, 2), (-2, 1)];

#[derive(Default, DynClone)]
#[dyn_clone(Constraint + Send)]
pub struct AntiKnightConstraint;

impl Constraint for AntiKnightConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        for row in 0..context.height() {
            for col in 0..context.width() {
                for (mut dr, mut dc) in KNIGHT_DELTAS {
                    dr += row as isize;
                    dc += col as isize;
                    if dr < 0
                        || dc < 0
                        || dr as usize >= context.height()
                        || dc as usize >= context.width()
                    {
                        continue;
                    }

                    solver.assert(
                        context.bools().alloc(
                            context
                                .get_cell(row, col)
                                ._eq(context.get_cell(dr as usize, dc as usize))
                                .not(),
                        ),
                    );
                }
            }
        }
    }
}

impl ConfigurableConstraint for AntiKnightConstraint {
    fn configure(&mut self, _ctx: &Context, _ui: &mut Ui) {
    }

    fn get_highlighted_cells(&mut self) -> Option<&mut Vec<Cell>> {
        None
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn name(&self) -> &'static str {
        "Anti-Knight"
    }
}
