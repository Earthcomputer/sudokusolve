mod anti_knight;
mod digit_definition;
mod given_digit;
mod killer_cage;
mod latin_square;
mod little_killer;
mod standard_boxes;

use crate::sudoku::SudokuContext;
use crate::ui::SudokuDrawContext;
use crate::{sudoku, DynClone};
use eframe::egui;

pub use digit_definition::*;
pub use given_digit::*;
pub use latin_square::*;
pub use standard_boxes::*;

use anti_knight::AntiKnightConstraint;
use killer_cage::KillerCageConstraint;
use little_killer::LittleKillerConstraint;

pub trait Constraint {
    fn apply<'a>(&self, solver: &z3::Solver, context: &'a SudokuContext);
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
    "Anti-Knight" => || Box::new(AntiKnightConstraint::default()),
    "Killer Cage" => || Box::new(KillerCageConstraint::default()),
    "Little Killer" => || Box::new(LittleKillerConstraint::default()),
};

pub fn make_default_constraint() -> Box<dyn ConfigurableConstraint> {
    Box::new(KillerCageConstraint::default())
}
