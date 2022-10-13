use eframe::egui;
use crate::constraint::{Constraint, DigitsConstraint, GivenDigitConstraint, LatinSquareConstraint, StandardBoxesConstraint};
use crate::sudoku;
use crate::sudoku::{SUDOKU_SIZE, SudokuContext};

struct SudokuWidget<'a> {
    width: usize,
    height: usize,
    given_digits: &'a mut [Option<i32>],
    selected_cell: &'a mut Option<sudoku::Cell>,
}

impl<'a> SudokuWidget<'a> {
    fn new(
        width: usize,
        height: usize,
        given_digits: &'a mut [Option<i32>],
        selected_cell: &'a mut Option<sudoku::Cell>,
    ) -> Self {
        assert_eq!(given_digits.len(), width * height);
        Self {
            width,
            height,
            given_digits,
            selected_cell,
        }
    }

    fn get_given_digit(&self, row: usize, col: usize) -> Option<i32> {
        assert!(row < self.width && col < self.height);
        self.given_digits[col + self.width * row]
    }

    fn set_given_digit(&mut self, row: usize, col: usize, digit: Option<i32>) {
        assert!(row < self.width && col < self.height);
        self.given_digits[col + self.width * row] = digit;
    }

    fn cell_rect(left: f32, top: f32, cell_size: f32, row: usize, col: usize) -> egui::Rect {
        egui::Rect::from_x_y_ranges(
            left + col as f32 * cell_size..=left + (col + 1) as f32 * cell_size,
            top + row as f32 * cell_size..=top + (row + 1) as f32 * cell_size,
        )
    }
}

impl<'a> egui::Widget for SudokuWidget<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let egui::Vec2 {
            x: mut width,
            y: mut height,
        } = ui.available_size();
        let cell_size = (width / self.width as f32).min(height / self.height as f32);
        width = cell_size * self.width as f32;
        height = cell_size * self.height as f32;
        ui.allocate_ui(egui::Vec2::new(width, height), |ui| {
            ui.set_min_width(width);
            ui.set_min_height(height);
            let egui::Rect {
                min: egui::Pos2 { x: left, y: top },
                ..
            } = ui.max_rect();
            for x in 0..=self.width {
                let mut stroke = ui.style().visuals.widgets.noninteractive.fg_stroke;
                stroke.width = if x % 3 == 0 { 3f32 } else { 1f32 };
                ui.painter()
                    .vline(left + x as f32 * cell_size, top..=top + height, stroke);
            }
            for y in 0..=self.height {
                let mut stroke = ui.style().visuals.widgets.noninteractive.fg_stroke;
                stroke.width = if y % 3 == 0 { 3f32 } else { 1f32 };
                ui.painter()
                    .hline(left..=left + width, top + y as f32 * cell_size, stroke);
            }
            if let Some(selected) = self.selected_cell {
                let mut stroke = ui.style().visuals.selection.stroke;
                stroke.width = 2f32;
                ui.painter().rect_stroke(
                    Self::cell_rect(left, top, cell_size, selected.row, selected.col).shrink(2.0),
                    2f32,
                    stroke,
                );
            }

            let mut clicked_cell = false;
            for row in 0..self.height {
                for col in 0..self.width {
                    if let Some(digit) = self.get_given_digit(row, col) {
                        let mut font = egui::FontSelection::Default.resolve(ui.style());
                        font.size = cell_size / ui.input().pixels_per_point * 0.8;
                        ui.painter().text(
                            Self::cell_rect(left, top, cell_size, row, col).center(),
                            egui::Align2::CENTER_CENTER,
                            digit,
                            font,
                            ui.style().visuals.widgets.active.text_color(),
                        );
                    }
                    if ui
                        .interact(
                            Self::cell_rect(left, top, cell_size, row, col),
                            ui.make_persistent_id((row, col)),
                            egui::Sense::click(),
                        )
                        .clicked()
                    {
                        *self.selected_cell = Some(sudoku::Cell::new(row, col));
                        clicked_cell = true;
                    }
                }
            }

            if !clicked_cell && ui.input().pointer.primary_released() {
                *self.selected_cell = None;
            }

            for event in &ui.input().events {
                match event {
                    egui::Event::Key {
                        key: egui::Key::ArrowUp,
                        pressed: true,
                        ..
                    } => match self.selected_cell.as_mut() {
                        Some(cell) => {
                            if cell.row == 0 {
                                cell.row = SUDOKU_SIZE - 1;
                            } else {
                                cell.row -= 1;
                            }
                        }
                        None => *self.selected_cell = Some(sudoku::Cell::new(0, 0)),
                    },
                    egui::Event::Key {
                        key: egui::Key::ArrowDown,
                        pressed: true,
                        ..
                    } => match self.selected_cell.as_mut() {
                        Some(cell) => cell.row = (cell.row + 1) % SUDOKU_SIZE,
                        None => *self.selected_cell = Some(sudoku::Cell::new(0, 0)),
                    },
                    egui::Event::Key {
                        key: egui::Key::ArrowLeft,
                        pressed: true,
                        ..
                    } => match self.selected_cell.as_mut() {
                        Some(cell) => {
                            if cell.col == 0 {
                                cell.col = SUDOKU_SIZE - 1;
                            } else {
                                cell.col -= 1;
                            }
                        }
                        None => *self.selected_cell = Some(sudoku::Cell::new(0, 0)),
                    },
                    egui::Event::Key {
                        key: egui::Key::ArrowRight,
                        pressed: true,
                        ..
                    } => match self.selected_cell.as_mut() {
                        Some(cell) => cell.col = (cell.col + 1) % SUDOKU_SIZE,
                        None => *self.selected_cell = Some(sudoku::Cell::new(0, 0)),
                    },
                    egui::Event::Key {
                        key: egui::Key::Delete | egui::Key::Backspace,
                        pressed: true,
                        ..
                    } => {
                        if let Some(cell) = *self.selected_cell {
                            self.set_given_digit(cell.row, cell.col, None);
                        }
                    }
                    egui::Event::Key {
                        key: egui::Key::Escape,
                        pressed: true,
                        ..
                    } => {
                        *self.selected_cell = None;
                    }
                    egui::Event::Key {
                        key, pressed: true, ..
                    } => {
                        let digit = match key {
                            egui::Key::Num1 => Some(1),
                            egui::Key::Num2 => Some(2),
                            egui::Key::Num3 => Some(3),
                            egui::Key::Num4 => Some(4),
                            egui::Key::Num5 => Some(5),
                            egui::Key::Num6 => Some(6),
                            egui::Key::Num7 => Some(7),
                            egui::Key::Num8 => Some(8),
                            egui::Key::Num9 => Some(9),
                            _ => None,
                        };
                        if let (Some(digit), Some(cell)) = (digit, *self.selected_cell) {
                            self.set_given_digit(cell.row, cell.col, Some(digit));
                        }
                    }
                    _ => {}
                }
            }
        })
            .response
    }
}

struct MyApp {
    grid: [Option<i32>; SUDOKU_SIZE * SUDOKU_SIZE],
    selected_cell: Option<sudoku::Cell>,
}

impl MyApp {
    fn new() -> MyApp {
        MyApp {
            grid: [None; SUDOKU_SIZE * SUDOKU_SIZE],
            selected_cell: None,
        }
    }

    fn solve(&mut self) -> z3::SatResult {
        let cfg = z3::Config::new();
        let ctx = z3::Context::new(&cfg);
        let sudoku = SudokuContext::create(ctx);
        let solver = z3::Solver::new(sudoku.ctx());

        let mut constraints: Vec<Box<dyn Constraint>> = Vec::new();
        constraints.push(Box::new(DigitsConstraint));
        constraints.push(Box::new(LatinSquareConstraint));
        constraints.push(Box::new(StandardBoxesConstraint));
        for row in 0..SUDOKU_SIZE {
            for col in 0..SUDOKU_SIZE {
                if let Some(digit) = self.grid[col + SUDOKU_SIZE * row] {
                    constraints.push(Box::new(GivenDigitConstraint {
                        row,
                        col,
                        value: digit,
                    }));
                }
            }
        }

        for constraint in &constraints {
            constraint.apply(&solver, &sudoku);
        }

        let result = solver.check();
        if result != z3::SatResult::Sat {
            return result;
        }

        let model = solver
            .get_model()
            .expect("The solver check should have passed");
        for row in 0..sudoku.height() {
            for col in 0..sudoku.width() {
                let result = model
                    .eval(sudoku.get_cell(row, col), true)
                    .unwrap()
                    .as_u64()
                    .unwrap() as i32;
                self.grid[col + row * SUDOKU_SIZE] = Some(result);
            }
        }

        return result;
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::canvas(&ctx.style()).inner_margin(10f32))
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    if ui.button("Solve").clicked() {
                        self.solve();
                    }
                    ui.add_space(5.0);
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        ui.heading("Sudoku Solver");
                        ui.add_space(5.0);
                        ui.add(SudokuWidget::new(
                            SUDOKU_SIZE,
                            SUDOKU_SIZE,
                            &mut self.grid,
                            &mut self.selected_cell,
                        ));
                    });
                });
            });
    }
}

pub fn run() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Sudoku Solver",
        options,
        Box::new(|ctx| {
            let mut new_style = (*ctx.egui_ctx.style()).clone();
            for font in new_style.text_styles.values_mut() {
                font.size *= 3.0;
            }
            ctx.egui_ctx.set_style(new_style);

            Box::new(MyApp::new())
        }),
    );
}
