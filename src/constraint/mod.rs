mod anti_knight;
mod diagonal;
mod digit_definition;
mod given_digit;
mod killer_cage;
mod kropki;
mod latin_square;
mod little_killer;
mod standard_boxes;
mod thermo;

use crate::sudoku::SudokuContext;
use crate::ui::SudokuDrawContext;
use crate::{sudoku, DynClone};
use eframe::egui;

pub use digit_definition::*;
pub use given_digit::*;
pub use latin_square::*;
pub use standard_boxes::*;

use anti_knight::AntiKnightConstraint;
use diagonal::DiagonalConstraint;
use killer_cage::KillerCageConstraint;
use kropki::{BlackKropkiConstraint, WhiteKropkiConstraint};
use little_killer::LittleKillerConstraint;
use thermo::ThermoConstraint;

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
    fn always_draw(&self) -> bool {
        false
    }
    fn draw_depth(&self) -> i32 {
        0
    }
}

pub static CONFIGURABLES: phf::Map<&'static str, fn() -> Box<dyn ConfigurableConstraint>> = phf::phf_map! {
    "Anti-Knight" => || Box::new(AntiKnightConstraint::default()),
    "Black Kropki Dot" => || Box::new(BlackKropkiConstraint::default()),
    "Diagonal" => || Box::new(DiagonalConstraint::default()),
    "Killer Cage" => || Box::new(KillerCageConstraint::default()),
    "Little Killer" => || Box::new(LittleKillerConstraint::default()),
    "Thermo" => || Box::new(ThermoConstraint::default()),
    "White Kropki Dot" => || Box::new(WhiteKropkiConstraint::default()),
};

pub fn make_default_constraint() -> Box<dyn ConfigurableConstraint> {
    Box::new(KillerCageConstraint::default())
}
