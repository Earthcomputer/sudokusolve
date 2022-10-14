use crate::sudoku::{SudokuContext, SUDOKU_SIZE};
use crate::{sudoku, DynClone};
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
}
