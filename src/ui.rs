use crate::constraint::{
    ConfigurableConstraint, Constraint, DigitDefinitionConstraint, GivenDigitConstraint,
    LatinSquareConstraint, StandardBoxesConstraint,
};
use crate::sudoku::{SudokuContext, SUDOKU_SIZE};
use crate::{color, constraint, sudoku};
use eframe::egui;

struct ConstraintUi {
    color: egui::Color32,
    constraint: Box<dyn ConfigurableConstraint>,
}

pub const CELL_PADDING: f32 = 3.0;

pub struct SudokuDrawContext<'a> {
    pub width: usize,
    pub height: usize,
    left: f32,
    top: f32,
    cell_size: f32,
    pub color: egui::Color32,
    pub painter: &'a egui::Painter,
    pub style: &'a egui::Style,
    default_draw: std::cell::Cell<bool>,
}

impl<'a> SudokuDrawContext<'a> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        width: usize,
        height: usize,
        left: f32,
        top: f32,
        cell_size: f32,
        color: egui::Color32,
        painter: &'a egui::Painter,
        style: &'a egui::Style,
    ) -> Self {
        Self {
            width,
            height,
            left,
            top,
            cell_size,
            color,
            painter,
            style,
            default_draw: std::cell::Cell::new(false),
        }
    }

    pub fn cell_rect(&self, row: usize, col: usize) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::Pos2::new(
                self.left + col as f32 * self.cell_size,
                self.top + row as f32 * self.cell_size,
            ),
            egui::Vec2::splat(self.cell_size),
        )
    }

    pub fn get_font(&self, cell_ratio: f32) -> egui::FontId {
        let mut font = egui::TextStyle::Body.resolve(self.style);
        font.size = (self.cell_size * cell_ratio).max(1.0);
        font
    }

    pub fn default_draw(&self) {
        self.default_draw.set(true);
    }
}

struct SudokuWidget<'a> {
    width: usize,
    height: usize,
    given_digits: &'a mut [Option<i32>],
    selected_cell: &'a mut Option<sudoku::Cell>,
    solution: &'a mut Option<Vec<i32>>,
    extra_constraints: &'a mut [ConstraintUi],
    selected_extra_constraint: Option<usize>,
}

impl<'a> SudokuWidget<'a> {
    fn new(
        width: usize,
        height: usize,
        given_digits: &'a mut [Option<i32>],
        selected_cell: &'a mut Option<sudoku::Cell>,
        solution: &'a mut Option<Vec<i32>>,
        extra_constraints: &'a mut [ConstraintUi],
        selected_extra_constraint: Option<usize>,
    ) -> Self {
        assert_eq!(given_digits.len(), width * height);
        Self {
            width,
            height,
            given_digits,
            selected_cell,
            solution,
            extra_constraints,
            selected_extra_constraint,
        }
    }

    fn get_given_digit(&self, row: usize, col: usize) -> Option<i32> {
        assert!(row < self.width && col < self.height);
        self.given_digits[col + self.width * row]
    }

    fn set_given_digit(&mut self, row: usize, col: usize, digit: Option<i32>) {
        assert!(row < self.width && col < self.height);
        self.given_digits[col + self.width * row] = digit;
        *self.solution = None;
    }

    fn cell_rect(left: f32, top: f32, cell_size: f32, row: usize, col: usize) -> egui::Rect {
        egui::Rect::from_x_y_ranges(
            left + col as f32 * cell_size..=left + (col + 1) as f32 * cell_size,
            top + row as f32 * cell_size..=top + (row + 1) as f32 * cell_size,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_digit(
        left: f32,
        top: f32,
        cell_size: f32,
        row: usize,
        col: usize,
        digit: i32,
        ui: &egui::Ui,
        color: egui::Color32,
    ) {
        let mut font = egui::FontSelection::Default.resolve(ui.style());
        font.size = cell_size * 0.8;
        ui.painter().text(
            Self::cell_rect(left, top, cell_size, row, col).center(),
            egui::Align2::CENTER_CENTER,
            digit,
            font,
            color,
        );
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

            let mut n_times_cell_constrained = vec![0; self.width * self.height];
            let mut depth_sorted_constraints: Vec<_> =
                self.extra_constraints.iter_mut().enumerate().collect();
            depth_sorted_constraints
                .sort_by_key(|(_, constraint)| -constraint.constraint.draw_depth());
            for (constraint_index, constraint) in depth_sorted_constraints {
                if self.selected_extra_constraint.contains(&constraint_index)
                    && !constraint.constraint.always_draw()
                {
                    continue;
                }

                let context = SudokuDrawContext::new(
                    self.width,
                    self.height,
                    left,
                    top,
                    cell_size,
                    constraint.color,
                    ui.painter(),
                    ui.style(),
                );
                constraint.constraint.draw(&context);
                if !context.default_draw.get() {
                    if let Some(cells) = constraint.constraint.get_highlighted_cells() {
                        for cell in cells {
                            if n_times_cell_constrained[cell.col + self.width * cell.row] == 0 {
                                n_times_cell_constrained[cell.col + self.width * cell.row] = 1;
                            }
                        }
                    }
                    continue;
                }

                if let Some(cells) = constraint.constraint.get_highlighted_cells() {
                    for cell in cells {
                        let mut cell_rect =
                            Self::cell_rect(left, top, cell_size, cell.row, cell.col);
                        n_times_cell_constrained[cell.col + self.width * cell.row] += 1;
                        let amt_to_shrink = ((2 * n_times_cell_constrained
                            [cell.col + self.width * cell.row])
                            as f32)
                            .min(cell_rect.width() * 0.5 - 1.0);
                        cell_rect = cell_rect.shrink(amt_to_shrink);
                        ui.painter().rect_stroke(
                            cell_rect,
                            2f32,
                            egui::Stroke::new(1f32, constraint.color),
                        );
                    }
                }
            }

            if let Some(selected_constraint) = self.selected_extra_constraint {
                let constraint = &mut self.extra_constraints[selected_constraint];
                if let Some(cells) = constraint.constraint.get_highlighted_cells() {
                    for (index, cell) in cells.iter().enumerate() {
                        let cell_rect = Self::cell_rect(left, top, cell_size, cell.row, cell.col)
                            .shrink(CELL_PADDING);
                        ui.painter().rect_stroke(
                            cell_rect,
                            2f32,
                            egui::Stroke::new(3f32, constraint.color),
                        );
                        ui.painter().text(
                            cell_rect.min + egui::Vec2::splat(CELL_PADDING),
                            egui::Align2::LEFT_TOP,
                            index + 1,
                            egui::FontSelection::Default.resolve(ui.style()),
                            constraint.color,
                        );
                    }
                }
            }

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

            let mut clicked_cell = false;
            for row in 0..self.height {
                for col in 0..self.width {
                    if let Some(digit) = self.get_given_digit(row, col) {
                        Self::draw_digit(
                            left,
                            top,
                            cell_size,
                            row,
                            col,
                            digit,
                            ui,
                            ui.style().visuals.widgets.active.text_color(),
                        );
                    } else if let Some(solution) = self.solution {
                        Self::draw_digit(
                            left,
                            top,
                            cell_size,
                            row,
                            col,
                            solution[col + SUDOKU_SIZE * row],
                            ui,
                            if ui.style().visuals.dark_mode {
                                egui::Color32::LIGHT_BLUE
                            } else {
                                egui::Color32::DARK_BLUE
                            },
                        );
                    }
                    let cell_interaction = ui.interact(
                        Self::cell_rect(left, top, cell_size, row, col),
                        ui.make_persistent_id((row, col)),
                        egui::Sense::click(),
                    );
                    if cell_interaction.clicked() {
                        *self.selected_cell = Some(sudoku::Cell::new(row, col));
                        clicked_cell = true;
                    } else if cell_interaction.clicked_by(egui::PointerButton::Secondary) {
                        if let Some(constraint_index) = self.selected_extra_constraint {
                            let constraint =
                                &mut self.extra_constraints[constraint_index].constraint;
                            let max_highlighted_cells = constraint.get_max_highlighted_cells();
                            if let Some(highlighted_cells) = constraint.get_highlighted_cells() {
                                if let Some(existing_index) = highlighted_cells
                                    .iter()
                                    .position(|c| c.row == row && c.col == col)
                                {
                                    highlighted_cells.remove(existing_index);
                                } else {
                                    if highlighted_cells.len() == max_highlighted_cells {
                                        highlighted_cells.remove(0);
                                    }
                                    highlighted_cells.push(sudoku::Cell::new(row, col));
                                }
                                *self.selected_cell = None;
                                clicked_cell = true;
                            }
                        }
                    }
                }
            }

            if let Some(selected) = self.selected_cell {
                let mut stroke = ui.style().visuals.selection.stroke;
                stroke.width = 2f32;
                ui.painter().rect_stroke(
                    Self::cell_rect(left, top, cell_size, selected.row, selected.col)
                        .shrink(CELL_PADDING),
                    2f32,
                    stroke,
                );
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

enum SolveResult {
    Ok,
    Unsolvable,
    TimedOut,
    InvalidInput,
}

impl SolveResult {
    fn message(&self) -> &'static str {
        match self {
            SolveResult::Ok => "",
            SolveResult::Unsolvable => "Unsolvable",
            SolveResult::TimedOut => "Solver timed out",
            SolveResult::InvalidInput => "Invalid input",
        }
    }
}

struct MyApp {
    grid: [Option<i32>; SUDOKU_SIZE * SUDOKU_SIZE],
    selected_cell: Option<sudoku::Cell>,
    solution: Option<Vec<i32>>,
    extra_constraints: Vec<ConstraintUi>,
    selected_constraint: Option<usize>,
    error_message: &'static str,
}

impl MyApp {
    fn new() -> MyApp {
        MyApp {
            grid: [None; SUDOKU_SIZE * SUDOKU_SIZE],
            selected_cell: None,
            solution: None,
            extra_constraints: Vec::new(),
            selected_constraint: None,
            error_message: "",
        }
    }

    fn solve(&mut self) -> SolveResult {
        if self
            .extra_constraints
            .iter()
            .any(|constraint| !constraint.constraint.is_valid())
        {
            return SolveResult::InvalidInput;
        }

        let mut cfg = z3::Config::new();
        cfg.set_param_value("timeout", "5000");
        let ctx = z3::Context::new(&cfg);
        let sudoku = SudokuContext::create(ctx);
        let solver = z3::Solver::new(sudoku.ctx());

        let mut constraints: Vec<Box<dyn Constraint>> = Vec::new();
        constraints.push(Box::new(DigitDefinitionConstraint));
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
        constraints.extend(
            self.extra_constraints
                .iter()
                .map(|constraint| constraint.constraint.dyn_clone()),
        );

        for constraint in &constraints {
            constraint.apply(&solver, &sudoku);
        }

        let result = solver.check();
        match result {
            z3::SatResult::Unsat => return SolveResult::Unsolvable,
            z3::SatResult::Unknown => return SolveResult::TimedOut,
            z3::SatResult::Sat => {}
        };

        let model = solver
            .get_model()
            .expect("The solver check should have passed");
        let mut solution = vec![0; SUDOKU_SIZE * SUDOKU_SIZE];
        for row in 0..sudoku.height() {
            for col in 0..sudoku.width() {
                let result = model
                    .eval(sudoku.get_cell(row, col), true)
                    .unwrap()
                    .as_u64()
                    .unwrap() as i32;
                solution[col + row * SUDOKU_SIZE] = result;
            }
        }
        self.solution = Some(solution);

        SolveResult::Ok
    }

    fn extra_constraints_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("constraint_list")
            .height_range(ui.available_height() / 3.0..=ui.available_height() / 3.0)
            .show_inside(ui, |ui| {
                if ui.button("Add Constraint").clicked() {
                    self.selected_constraint = Some(self.extra_constraints.len());
                    let mut colors_to_avoid: Vec<_> =
                        self.extra_constraints.iter().map(|c| c.color).collect();
                    colors_to_avoid.extend_from_slice(&[ui
                        .style()
                        .visuals
                        .widgets
                        .active
                        .text_color()]);
                    let color = color::next_distinguishable_color(
                        &colors_to_avoid,
                        ui.style().visuals.extreme_bg_color,
                    );
                    self.extra_constraints.push(ConstraintUi {
                        color,
                        constraint: constraint::make_default_constraint(),
                    });
                }
                ui.add_space(5.0);
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for constraint_index in 0..self.extra_constraints.len() {
                            let mut should_break = false;
                            ui.horizontal(|ui| {
                                let selected = self.selected_constraint.contains(&constraint_index);
                                let mut text = egui::RichText::new(
                                    self.extra_constraints[constraint_index].constraint.name(),
                                );
                                if !selected {
                                    text =
                                        text.color(self.extra_constraints[constraint_index].color);
                                }
                                if ui.selectable_label(selected, text).clicked() {
                                    self.selected_constraint = Some(constraint_index);
                                }
                                if ui
                                    .button(
                                        egui::RichText::new("Delete")
                                            .text_style(egui::TextStyle::Small),
                                    )
                                    .clicked()
                                {
                                    self.selected_constraint = None;
                                    self.extra_constraints.remove(constraint_index);
                                    should_break = true;
                                }
                            });
                            if should_break {
                                break;
                            }
                        }
                    });
            });

        ui.add_space(10.0);

        if let Some(selected_constraint) = self.selected_constraint {
            let mut constraint = &mut self.extra_constraints[selected_constraint];

            let mut constraint_name = constraint.constraint.name();
            egui::ComboBox::from_id_source("constraint_type")
                .selected_text(constraint.constraint.name())
                .show_ui(ui, |ui| {
                    let mut constraints: Vec<_> =
                        constraint::CONFIGURABLES.keys().copied().collect();
                    constraints.sort();
                    for constraint in constraints {
                        ui.selectable_value(&mut constraint_name, constraint, constraint);
                    }
                });
            if constraint_name != constraint.constraint.name() {
                if let Some(constraint_creator) = constraint::CONFIGURABLES.get(constraint_name) {
                    let mut new_constraint = constraint_creator();
                    let new_max_highlighted = new_constraint.get_max_highlighted_cells();
                    if let (Some(new_highlighted), Some(old_highlighted)) = (
                        new_constraint.get_highlighted_cells(),
                        constraint.constraint.get_highlighted_cells(),
                    ) {
                        if new_max_highlighted < old_highlighted.len() {
                            new_highlighted
                                .extend_from_slice(&old_highlighted[..new_max_highlighted]);
                        } else {
                            new_highlighted.extend_from_slice(old_highlighted);
                        }
                    }
                    let color = constraint.color;
                    self.extra_constraints[selected_constraint] = ConstraintUi {
                        color,
                        constraint: new_constraint,
                    };
                    constraint = &mut self.extra_constraints[selected_constraint];
                }
            }

            ui.add_space(5.0);

            if constraint.constraint.get_highlighted_cells().is_some() {
                if constraint.constraint.get_max_highlighted_cells() == 1 {
                    ui.label("Right click to select the cell for this constraint");
                } else {
                    ui.label("Right click to add cells to this constraint");
                }
                ui.add_space(5.0);
            }
            constraint.constraint.configure(ctx, ui);
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("constraints_panel").show(ctx, |ui| {
            self.extra_constraints_ui(ctx, ui);
            ui.set_min_width(ctx.available_rect().width() / 3.0);
        });
        egui::CentralPanel::default()
            .frame(egui::Frame::canvas(&ctx.style()).inner_margin(10f32))
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .button(egui::RichText::new("Solve").font({
                                let mut font = egui::TextStyle::Heading.resolve(ui.style());
                                font.size *= 2.0;
                                font
                            }))
                            .clicked()
                        {
                            let result = self.solve();
                            self.error_message = result.message();
                        }
                        ui.heading(
                            egui::RichText::new(self.error_message)
                                .color(ui.style().visuals.error_fg_color),
                        );
                    });
                    ui.add_space(5.0);
                    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        ui.heading(egui::RichText::new("Sudoku Solver").font({
                            let mut font = egui::TextStyle::Heading.resolve(ui.style());
                            font.size *= 3.0;
                            font
                        }));
                        ui.add_space(5.0);
                        egui::CentralPanel::default()
                            .frame(
                                egui::Frame::canvas(&ctx.style())
                                    .inner_margin(20f32)
                                    .stroke(egui::Stroke::none()),
                            )
                            .show_inside(ui, |ui| {
                                ui.horizontal_centered(|ui| {
                                    ui.add(SudokuWidget::new(
                                        SUDOKU_SIZE,
                                        SUDOKU_SIZE,
                                        &mut self.grid,
                                        &mut self.selected_cell,
                                        &mut self.solution,
                                        &mut self.extra_constraints,
                                        self.selected_constraint,
                                    ));
                                });
                            });
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
        Box::new(|_| Box::new(MyApp::new())),
    );
}
