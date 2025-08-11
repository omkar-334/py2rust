//! A simple calculator application built with Rust and egui,
//! converted from a Python Tkinter project.

use eframe::egui;

/// Main function to set up and run the eframe application.
fn main() -> Result<(), eframe::Error> {
    // Configure the native window options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([380.0, 300.0])
            .with_resizable(false),
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "Simple Rust Calculator",
        options,
        Box::new(|_cc| Box::new(CalculatorApp::default())),
    )
}

/// Enum to represent the different calculation operations.
enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// Struct to hold the result of a calculation.
struct OperationResult {
    op_name: String,
    value: f64,
    color: egui::Color32,
}

/// The main application struct that holds the state.
struct CalculatorApp {
    num1_str: String,
    num2_str: String,
    operation_result: Option<OperationResult>,
    show_author_window: bool,
    error_message: Option<String>,
}

impl Default for CalculatorApp {
    /// Creates a new `CalculatorApp` with default values.
    fn default() -> Self {
        Self {
            num1_str: String::new(),
            num2_str: String::new(),
            operation_result: None,
            show_author_window: false,
            error_message: None,
        }
    }
}

impl CalculatorApp {
    /// Performs a calculation based on the given operation.
    ///
    /// It parses the input strings, performs the calculation,
    /// and updates the application state with the result or an error message.
    fn perform_operation(&mut self, op: Operation) {
        // Clear previous results and errors
        self.error_message = None;
        self.operation_result = None;

        // Trim whitespace and parse the input strings into f64 numbers.
        let num1_res = self.num1_str.trim().parse::<f64>();
        let num2_res = self.num2_str.trim().parse::<f64>();

        match (num1_res, num2_res) {
            (Ok(n1), Ok(n2)) => {
                // Both numbers are valid, perform the operation.
                let (op_name, value, color) = match op {
                    Operation::Add => ("Summation", n1 + n2, egui::Color32::from_rgb(255, 69, 0)), // Red-Orange
                    Operation::Subtract => ("Subtraction", n1 - n2, egui::Color32::from_rgb(60, 179, 113)), // Medium Sea Green
                    Operation::Multiply => ("Multiplication", n1 * n2, egui::Color32::from_rgb(30, 144, 255)), // Dodger Blue
                    Operation::Divide => {
                        if n2 == 0.0 {
                            self.error_message = Some("Error: Division by zero.".to_string());
                            return;
                        }
                        ("Division", n1 / n2, egui::Color32::from_rgb(255, 215, 0)) // Gold
                    }
                };

                self.operation_result = Some(OperationResult {
                    op_name: op_name.to_string(),
                    value,
                    color,
                });
            }
            _ => {
                // One or both numbers are invalid.
                self.error_message = Some(
                    "Invalid number input.\nPlease enter valid numbers like 123, -45.6, or .789"
                        .to_string(),
                );
            }
        }
    }

    /// Renders the author information window if `show_author_window` is true.
    fn show_author_dialog(&mut self, ctx: &egui::Context) {
        if self.show_author_window {
            egui::Window::new("Author Information")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("Pranta Sarker");
                        ui.label("Batch: 6th");
                        ui.label("Department: CSE");
                        ui.label("North East University Bangladesh");
                        ui.add_space(10.0);
                        if ui.button("Close").clicked() {
                            self.show_author_window = false;
                        }
                    });
                });
        }
    }

    /// Renders an error message window if `error_message` is Some.
    fn show_error_dialog(&mut self, ctx: &egui::Context) {
        if let Some(error) = self.error_message.clone() {
            let mut is_open = true;
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .open(&mut is_open)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.colored_label(egui::Color32::RED, &error);
                        ui.add_space(10.0);
                        if ui.button("OK").clicked() {
                            self.error_message = None;
                        }
                    });
                });
            
            // If the user closes the window via the 'x' button
            if !is_open {
                self.error_message = None;
            }
        }
    }
}

/// Implement the `eframe::App` trait to define the application's UI and logic.
impl eframe::App for CalculatorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Main panel with a centered vertical layout
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(15.0);
                ui.heading("Simple Rust Calculator");
                ui.add_space(25.0);

                // Input fields
                let input_width = 150.0;
                ui.add(
                    egui::TextEdit::singleline(&mut self.num1_str)
                        .hint_text("Enter first number")
                        .desired_width(input_width),
                );
                ui.add_space(5.0);
                ui.add(
                    egui::TextEdit::singleline(&mut self.num2_str)
                        .hint_text("Enter second number")
                        .desired_width(input_width),
                );
                ui.add_space(20.0);

                // Result display area
                ui.separator();
                ui.add_space(10.0);
                if let Some(ref result) = self.operation_result {
                    ui.colored_label(result.color, &result.op_name);
                    ui.colored_label(result.color, egui::RichText::new(result.value.to_string()).size(20.0));
                } else {
                    ui.label("Result will be shown here");
                    ui.label(egui::RichText::new("0").size(20.0));
                }
                ui.add_space(10.0);
                ui.separator();
            });

            // Place operation buttons in a specific area to control layout
            egui::Area::new("operation_buttons".into())
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 80.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("+").size(20.0)).clicked() {
                            self.perform_operation(Operation::Add);
                        }
                        ui.add_space(10.0);
                        if ui.button(egui::RichText::new("-").size(20.0)).clicked() {
                            self.perform_operation(Operation::Subtract);
                        }
                        ui.add_space(10.0);
                        if ui.button(egui::RichText::new("*").size(20.0)).clicked() {
                            self.perform_operation(Operation::Multiply);
                        }
                        ui.add_space(10.0);
                        if ui.button(egui::RichText::new("/").size(20.0)).clicked() {
                            self.perform_operation(Operation::Divide);
                        }
                    });
                });

            // Place the author button at the bottom center
            egui::Area::new("author_button_area".into())
                .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -10.0))
                .show(ctx, |ui| {
                    if ui.button("Author").clicked() {
                        self.show_author_window = true;
                    }
                });
        });

        // Show modal dialogs if needed
        self.show_author_dialog(ctx);
        self.show_error_dialog(ctx);
    }
}