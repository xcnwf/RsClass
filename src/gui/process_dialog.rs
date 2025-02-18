/***
 * Process selection dialog
 * Inspired by egui_file
 * 
*/

use sysinfo::{Pid, System, ProcessesToUpdate::All};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum State {
    Open,       // Visible
    Closed,     // Destroyed
    Selected,   // Process is selected
    Cancelled   // 
}

pub struct ProcessDialog {
    state: State,
    system: System,
    selected_process_id: Option<Pid>,
}

impl ProcessDialog {
    pub fn new() -> Self {
        ProcessDialog {
            state: State::Closed,
            system: Default::default(),
            selected_process_id: None,
        }
    }

    pub fn selected(&self) -> bool {
        self.state == State::Selected
    }

    pub fn state(&self) -> State {
        self.state
    }
    pub fn pid(&self) -> Option<Pid> {
        self.selected_process_id
    }

    fn refresh(&mut self) {
        self.system.refresh_processes(All, true);
    }

    // Opens the dialog
    pub fn open(&mut self) {
        self.state = State::Open;
        self.refresh();
    }

    pub fn show(&mut self, ctx: &egui::Context) -> &Self {
        self.state = match self.state {
            State::Open => {
                if ctx.input(|state| state.key_pressed(egui::Key::Escape)) {
                    self.state = State::Cancelled;
                }

                let mut is_open = true;
                self.ui(ctx, &mut is_open);
                match is_open {
                    true => self.state,
                    false => State::Cancelled,
                }
            },
            _ => State::Closed,
        };

        self
    }

    fn ui(&mut self, ctx: &egui::Context, is_open: &mut bool) {
        let window = egui::Window::new("Process Selection")
            .open(is_open)
            //.default_size(self.default_size)
            .resizable(true)
            .collapsible(false);

        window.show(ctx, |ui| {
            ui.ctx().move_to_top(ui.layer_id());
            self.ui_in_window(ui)
        });
    }

    fn ui_in_window(&mut self, ui: &mut egui::Ui) {
        ui.heading("Select a process");
        egui::ScrollArea::vertical().show(ui, |ui | {
            for (pid, process) in self.system.processes() {
                let label_response = ui.selectable_value(
                    &mut self.selected_process_id,
                    Some(*pid), 
                    format!("{:10} | {}", pid, process.name().to_str().unwrap_or_default())
                );
                if label_response.clicked() {
                    self.selected_process_id = Some(pid.to_owned())
                }
                if label_response.double_clicked() {
                    self.selected_process_id = Some(pid.to_owned());
                    self.state = State::Selected;
                }
            }
        });
        
        if ui.button("Refresh").clicked() {
            self.refresh();
        }
        ui.horizontal(|ui| {
            let ok_button = egui::Button::new("Ok");
            if ui.add_enabled(self.selected_process_id.is_some(), ok_button).clicked() {
                self.state = State::Selected;
            }
            if ui.button("Close").clicked() {
                self.state = State::Cancelled;
            }
        });
    }
}