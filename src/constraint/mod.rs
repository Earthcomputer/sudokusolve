mod anti_knight;
mod arrow;
mod diagonal;
mod digit_definition;
mod german_whisper;
mod given_digit;
mod killer_cage;
mod kropki;
mod latin_square;
mod little_killer;
mod palindrome;
mod parity;
mod renban;
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
use arrow::ArrowConstraint;
use diagonal::DiagonalConstraint;
use german_whisper::GermanWhisperConstraint;
use killer_cage::KillerCageConstraint;
use kropki::{BlackKropkiConstraint, WhiteKropkiConstraint};
use little_killer::LittleKillerConstraint;
use palindrome::PalindromeConstraint;
use parity::ParityConstraint;
use renban::RenbanConstraint;
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
    "Anti-Knight" => || Box::<AntiKnightConstraint>::default(),
    "Arrow" => || Box::<ArrowConstraint>::default(),
    "Black Kropki Dot" => || Box::<BlackKropkiConstraint>::default(),
    "Diagonal" => || Box::<DiagonalConstraint>::default(),
    "German Whisper" => || Box::<GermanWhisperConstraint>::default(),
    "Killer Cage" => || Box::<KillerCageConstraint>::default(),
    "Little Killer" => || Box::<LittleKillerConstraint>::default(),
    "Palindrome" => || Box::<PalindromeConstraint>::default(),
    "Parity" => || Box::<ParityConstraint>::default(),
    "Renban" => || Box::<RenbanConstraint>::default(),
    "Thermo" => || Box::<ThermoConstraint>::default(),
    "White Kropki Dot" => || Box::<WhiteKropkiConstraint>::default(),
};

pub fn make_default_constraint() -> Box<dyn ConfigurableConstraint> {
    Box::<KillerCageConstraint>::default()
}
