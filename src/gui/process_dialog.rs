/***
 * Process selection dialog
 * Inspired by egui_file
 * 
*/

use std::{cell::RefCell, rc::Rc};

use egui::RichText;
use sysinfo::{Pid, System, ProcessesToUpdate::All};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum State {
    Open,       // Visible
    Closed,     // Destroyed
    Selected(Pid),   // Process is selected
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
        match self.state {
            State::Selected(_) => true,
            _ => false
        }
    }

    pub fn cancel(&mut self) {
        self.state = State::Cancelled;
    }

    pub fn state(&self) -> State {
        self.state
    }
    pub fn pid(&self) -> Option<Pid> {
        match self.state {
            State::Selected(pid) => Some(pid.clone()),
            _ => None
        }
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
                let mut is_open = true;
                self.ui(ctx, &mut is_open);
                
                if is_open {
                    self.state
                } else {
                    State::Cancelled
                }
            },
            _ => State::Closed,
        };

        self
    }

    fn ui(&mut self, ctx: &egui::Context, is_open: &mut bool) {
        let window = egui::Modal::new("Process Selection Window".into());
        if window.show(ctx, |ui| {
            self.ui_in_window(ui)
        }).should_close(){
            *is_open = false;
        }
    }

    fn ui_in_window(&mut self, ui: &mut egui::Ui) {
        ui.heading("Select a process");
        let rect = ui.input(|is| is.viewport().inner_rect);

        egui::ScrollArea::vertical()
            .max_height(rect.map(|r| r.height()/2.0).unwrap_or(f32::MAX))
            .show(ui, |ui | {
            
            for (pid, process) in self.system.processes() {
                let label_response = ui.selectable_value(
                    &mut self.selected_process_id,
                    Some(*pid), 
                    RichText::new(format!("{:>6} | {}", pid.as_u32(), process.name().to_str().unwrap_or_default())).monospace()
                );
                if label_response.clicked() {
                    self.selected_process_id = Some(pid.to_owned())
                }
                if label_response.double_clicked() {
                    self.selected_process_id = Some(pid.to_owned());
                    self.state = State::Selected(pid.to_owned());
                }
            }
        });
        
        ui.add_space(10.0);

        let shared_self = Rc::new(RefCell::new(self));
        let shared_self_clone = shared_self.clone();

        egui::Sides::new().show(ui, 
            |ui| {
                if ui.button("Refresh").clicked() {
                    shared_self.borrow_mut().refresh();
                }
            },
            |ui| {
                ui.horizontal(|ui| {
                    let mut x = shared_self_clone.borrow_mut();
                    let ok_button = egui::Button::new("Ok");
                    if ui.add_enabled(x.selected_process_id.is_some(), ok_button).clicked() {
                        if let Some(pid) = x.selected_process_id {
                            x.state = State::Selected(pid.to_owned());
                        }
                    }
                    if ui.button("Close").clicked() {
                        x.state = State::Cancelled;
                    }
                });
            }
        );
    }
}