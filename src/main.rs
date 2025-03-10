use eframe::egui;
use egui::ahash::HashMap;
use egui::{Frame, Style};
use serde::{Deserialize, Serialize};
use sysinfo::{System, RefreshKind, ProcessRefreshKind};
use std::ops::DerefMut;
use std::path::{PathBuf, Path};
use std::sync::{Arc, Mutex};

use rs_class::{typing::*, ops::*};

mod gui;
use gui::{ProcessDialog, State as GuiState};

fn main() {
    let native_options = eframe::NativeOptions::default();
    let window_name = format!("RsClass - {}", env!("CARGO_PKG_VERSION"));
    eframe::run_native(&window_name, native_options, Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc))))).expect("eframe should run");
}

#[derive(Default)]
struct MyEguiApp {
    root_element: StructDataType,
    system: System,
    selected_process: Option<Process>,
    state: State,

    // dialogs
    process_dialog: Option<ProcessDialog>,
    closing_dialog: bool,
    save_file_dialog: egui_file_dialog::FileDialog,
    
    // file saving
    save_file_location: Option<PathBuf>,
    is_dirty: bool,
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
enum State {
    #[default]
    Normal,
    Load,
    Save,
    SaveAndQuit,
    Quit,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut s = Self::default();
        let health = IntegerDataType::default();
        let e = StructEntry::new("Health".into(), health.into());
        s.root_element.push_entry(e);

        let system = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));
        println!("Got {} processes.", system.processes().len());
        for (pid, p) in system.processes().iter().take(5) {
            println!("{} : {}", pid, p.name().to_str().unwrap_or_default());
        }
        s.system = system;
        s
    }

    fn save_to_file(&self) -> Result<(), String> {
        let file = std::fs::File::create(self.save_file_location.as_ref().ok_or("please select a save location")?).map_err(|e| e.to_string())?;
        let r = ron::ser::to_writer_pretty(file, &self.root_element, ron::ser::PrettyConfig::default())
            .map_err(|e| e.to_string());
        r
    }

    fn quit(&mut self, ctx: &egui::Context) {
        self.state = State::Quit;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for closing
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested && self.state != State::Quit &&self.is_dirty {
            self.closing_dialog = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        if self.closing_dialog {
            egui::Modal::new("close_unsaved_dialog".into()).show(ctx, 
            |ui| {
                ui.heading("You have unsaved changes");
                ui.horizontal(|ui| {
                    if ui.button("Quit w/o saving").clicked() {
                        self.closing_dialog = false;
                        self.quit(ctx);
                    }
                    if ui.button("Save & Quit").clicked() {
                        self.state = State::SaveAndQuit;
                        self.closing_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.closing_dialog = false;
                    }
                })
            });
        }

        self.save_file_dialog.update(ctx);

        // Handle save requests
        if self.state == State::SaveAndQuit || self.state == State::Save {
            //Are the changes saved ?
            if self.is_dirty {
                if self.save_file_location.is_some() {
                    match MyEguiApp::save_to_file(self) {
                        Ok(()) => {
                            self.is_dirty = false;
                            if self.state == State::SaveAndQuit {
                                self.quit(ctx);
                            }
                        }
                        Err(e) => {
                            println!("ERROR: could not save: {}",e);
                            self.state = State::Normal;
                        }
                    }
                } else {
                    if self.save_file_dialog.state() == egui_file_dialog::DialogState::Closed {
                        self.save_file_dialog
                            .save_file();
                    }

                    self.save_file_location = self.save_file_dialog.take_picked().map(|p| p.to_path_buf());
                    if self.save_file_dialog.state() == egui_file_dialog::DialogState::Cancelled {
                        println!("User did not choose save file, saving is cancelled");
                        self.state = State::Normal;
                    }
                }
            } else {
                if self.state == State::SaveAndQuit {
                    self.quit(ctx);
                } else {
                    self.state = State::Normal;
                }
            }
        }

        // Process Selection Window
        if let Some(pd) = self.process_dialog.as_mut() {
            pd.show(ctx);
            match pd.state() {
                GuiState::Closed => self.process_dialog = None,
                GuiState::Selected(pid) => {
                    self.selected_process = Some(Process::new(pid));
                    self.process_dialog = None;
                }
                _ => {}
            }
        }

        // GUI Interface
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("open process").clicked() {
                    let mut dialog = ProcessDialog::new();
                    dialog.open();
                    self.process_dialog = Some(dialog);
                };
                if ui.button("load").clicked() {
                    self.state = State::Load;
                };
                let save_button = egui::Button::new("save");
                if ui.add_enabled(self.is_dirty, save_button).clicked() {
                    self.state = State::Save;
                };
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.label(format!("App state: {:?}", self.state));
            ui.label(format!("Dirty? {}", self.is_dirty));
            ui.label(format!("File location: {:?}", self.save_file_location));
            ui.label(format!("Selected process: {}", self.selected_process.as_ref().and_then(|p| self.system.process(p.pid())).and_then(|p| p.name().to_str()).unwrap_or("None")));
            let pstatus = self.process_dialog.as_ref().map(|pd| pd.state());
            ui.label(format!("Process window status: {:?}", pstatus));


            if let Some(GuiState::Selected(pid)) = pstatus {
                self.system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
                if let Some(p) = self.system.process(pid) {
                    
                } else {
                    self.selected_process = None
                }
            }
        });
    }
}