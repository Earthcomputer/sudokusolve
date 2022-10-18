use crate::constraint::{ConfigurableConstraint, Constraint};
use crate::sudoku;
use crate::sudoku::SudokuContext;
use eframe::egui;
use eframe::egui::{Context, Ui};
use macros::DynClone;
use z3::ast::Ast;
use z3::Solver;
use crate::ui::SudokuDrawContext;

#[derive(Clone, Eq, PartialEq)]
enum Parity {
    Odd,
    Even,
}

#[derive(DynClone)]
#[dyn_clone(Constraint + Send)]
pub struct ParityConstraint {
    parity: Parity,
    cells: Vec<sudoku::Cell>,
}

impl Default for ParityConstraint {
    fn default() -> Self {
        Self {
            parity: Parity::Odd,
            cells: Vec::new(),
        }
    }
}

impl Constraint for ParityConstraint {
    fn apply<'a>(&self, solver: &Solver, context: &'a SudokuContext) {
        let is_even = context
            .get_cell(self.cells[0].row, self.cells[0].col)
            .modulo(context.const_int(2))
            ._eq(context.const_int(0));
        match self.parity {
            Parity::Even => solver.assert(context.bools().alloc(is_even)),
            Parity::Odd => solver.assert(context.bools().alloc(is_even.not())),
        }
    }
}

impl ConfigurableConstraint for ParityConstraint {
    fn configure(&mut self, _ctx: &Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Parity");
            egui::ComboBox::from_id_source("select_parity")
                .selected_text(match self.parity {
                    Parity::Odd => "Odd",
                    Parity::Even => "Even",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.parity, Parity::Odd, "Odd");
                    ui.selectable_value(&mut self.parity, Parity::Even, "Even");
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
        self.cells.len() == 1
    }

    fn name(&self) -> &'static str {
        "Parity"
    }

    fn draw(&self, context: &SudokuDrawContext) {
        if self.cells.is_empty() {
            context.default_draw();
            return;
        }

        for cell in &self.cells {
            let ratio = 0.8;
            let rect = context.cell_rect(cell.row, cell.col);
            match self.parity {
                Parity::Odd => context.painter.circle_filled(rect.center(), rect.width() * ratio * 0.5, context.color),
                Parity::Even => context.painter.rect_filled(rect.shrink(rect.width() * (1.0 - ratio) * 0.5), 0f32, context.color),
            }
        }
    }

    fn draw_depth(&self) -> i32 {
        10
    }
}
