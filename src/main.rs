use eframe::egui;
use serde::{Deserialize, Serialize};
use gui::type_selection_dialog::{self, TypeSelectionDialog};
use sysinfo::{System, RefreshKind, ProcessRefreshKind};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{PathBuf, Path};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use rs_class::{typing::*, ops::*};

mod gui;
use gui::process_dialog::{ProcessDialog, State as PDState};

fn main() {
    let native_options = eframe::NativeOptions::default();
    let window_name = format!("RsClass - {}", env!("CARGO_PKG_VERSION"));
    eframe::run_native(&window_name, native_options, Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc))))).expect("eframe should run");
}

#[derive(Default)]
struct MyEguiApp {
    struct_tabs: Vec<StructDataType>,
    system: System,

    // type system
    type_aliases: Rc<RefCell<HashMap<String, DataTypeEnum>>>,
    selected_type: Option<String>,

    selected_process: Option<Process>,
    state: State,

    // dialogs
    process_dialog: Option<ProcessDialog>,
    type_selection_dialog: Option<TypeSelectionDialog>,
    closing_dialog: bool,
    file_dialog: Option<egui_file_dialog::FileDialog>,
    save_load_dialog: bool,
    
    // file saving
    save_file_location: Option<PathBuf>,
    is_dirty: bool,
}

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
enum State {
    #[default]
    Normal,
    Load,
    SaveAndLoad,
    Save,
    SaveAndQuit,
    Quit,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut s = Self::default();
        let health = IntegerDataType::default();

        let mut type_aliases = s.type_aliases.borrow_mut();

        // add default datatypes
        type_aliases.insert(String::from("Int"), IntegerDataType::default().into());
        type_aliases.insert(String::from("UInt"), IntegerDataType::default().with_signed(false).into());
        type_aliases.insert(String::from("DWORD"), IntegerDataType::default().with_hex(true).into());
        type_aliases.insert(String::from("WORD"), IntegerDataType::default().with_hex(true).with_size(IntSize::Integer16).into());
        type_aliases.insert(String::from("Char"), IntegerDataType::default().with_size(IntSize::Integer8).into());
        type_aliases.insert(String::from("UChar"), IntegerDataType::default().with_size(IntSize::Integer8).with_signed(false).into());
        type_aliases.insert(String::from("CStr"), StrDataType::default().into());
        type_aliases.insert(String::from("Bool"), BooleanDataType::default().into());
        type_aliases.insert(String::from("Float"), FloatDataType::default().into());
        type_aliases.insert(String::from("Double"), FloatDataType::default().with_precision(FloatPrecision::Double).into());
        drop(type_aliases);

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
        ron::ser::to_writer_pretty(file, &self.struct_tabs, ron::ser::PrettyConfig::default())
            .map_err(|e| e.to_string())
    }

    fn load_from_file(&mut self) -> Result<(), String> {
        let file = std::fs::File::open(self.save_file_location.as_ref().ok_or("No file path for load available")?).map_err(|e| e.to_string())?;
        self.struct_tabs = ron::de::from_reader(file).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn quit(&mut self, ctx: &egui::Context) {
        self.state = State::Quit;
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Close if not in file dialog, and if not dirty
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested {
            match self.state {
                State::Save | State::SaveAndQuit | State::Load | State::SaveAndLoad => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                }
                State::Normal => if self.is_dirty {
                    self.closing_dialog = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                }
                _ => {}
            }
        }

        /* DIALOGS */

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

        // Handle save requests
        if self.state == State::SaveAndQuit || self.state == State::Save || self.state == State::SaveAndLoad {
            //Are the changes saved ?
            if self.is_dirty {
                if self.save_file_location.is_some() {
                    match MyEguiApp::save_to_file(self) {
                        Ok(()) => {
                            self.is_dirty = false;
                            if self.state == State::SaveAndQuit {
                                self.quit(ctx);
                            } else if self.state == State::SaveAndLoad {
                                self.state = State::Load;
                            }
                        }
                        Err(e) => {
                            println!("ERROR: Could not save: {}",e);
                            self.state = State::Normal;
                        }
                    }
                } else {
                    if let Some(fd) = self.file_dialog.as_mut() {
                        use egui_file_dialog::DialogState::*;
                        match fd.state() {
                            Open => {fd.update(ctx);},
                            Closed | Cancelled => {
                                println!("User did not choose save file, saving is cancelled");
                                self.state = State::Normal;
                                self.file_dialog = None;
                            },
                            Picked(_p) => {
                                self.save_file_location = fd.take_picked().map(|p| p.to_path_buf());
                                self.file_dialog = None;
                            },
                            PickedMultiple(_) => unreachable!()
                        }
                    } else {
                        let mut fd: egui_file_dialog::FileDialog = egui_file_dialog::FileDialog::new();
                        fd.config_mut().default_file_name = self
                                .selected_process
                                .as_ref()
                                .and_then(|p| self.system.process(p.pid()))
                                .and_then(|p| p.name().to_str())
                                .map(|s| PathBuf::from(s))
                                .and_then(|path| path.with_extension("rsclass").to_str().map(ToOwned::to_owned))
                                .unwrap_or("new_project.rsclass".into());

                        if let Some(p) = dirs::document_dir() {
                            fd.config_mut().initial_directory = p;
                        }
                        fd.save_file();
                        self.file_dialog = Some(fd);
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

        if self.save_load_dialog {
            egui::Modal::new("save_load_dialog".into()).show(ctx, 
            |ui| {
                ui.heading("Do you wish to save your changes ?");
                ui.horizontal(|ui| {
                    if ui.button("Discard changes").clicked() {
                        self.save_load_dialog = false;
                        // force discard changes
                        self.is_dirty = false;
                        self.state = State::Load;
                    }
                    if ui.button("Save changes").clicked() {
                        self.state = State::SaveAndLoad;
                        self.save_load_dialog = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.save_load_dialog = false;
                        self.state = State::Normal;
                    }
                })
            });
        }

        if self.state == State::Load {
            // The user has already opened/saved a project
            if self.is_dirty && self.save_file_location.is_some() {
                self.state = State::SaveAndLoad;
            } else {
                if let Some(fd) = self.file_dialog.as_mut() {
                    use egui_file_dialog::DialogState::*;
                    match fd.state() {
                        Open => {fd.update(ctx);},
                        Closed | Cancelled => {
                            println!("User did not choose a file, loading is cancelled.");
                            self.state = State::Normal;
                            self.file_dialog = None;
                        },
                        Picked(_p) => {
                            self.save_file_location = fd.take_picked().map(|p| p.to_path_buf());
                            match self.load_from_file() {
                                Ok(()) => {
                                    self.is_dirty = false;
                                    self.file_dialog = None;
                                    self.state = State::Normal;
                                    println!("Loaded from file!")
                                },
                                Err(e) => {
                                    println!("ERROR: Could not load: {}",e);
                                }
                            };
                        },
                        PickedMultiple(_) => unreachable!()
                    }
                } else {
                    let mut fd: egui_file_dialog::FileDialog = egui_file_dialog::FileDialog::new()
                        .add_file_filter("rsclass", Arc::new(|path| path.extension().map(|ext| ext == "rsclass").unwrap_or_default()))
                        .default_file_filter("rsclass");
                    if let Some(file_name) = self
                        .selected_process
                        .as_ref()
                        .and_then(|p| self.system.process(p.pid()))
                        .and_then(|p| p.name().to_str())
                        .map(|s| PathBuf::from(s))
                        .and_then(|path| path.with_extension("rsclass").to_str().map(ToOwned::to_owned))
                    {
                        fd.config_mut().default_file_name = file_name;
                    }
                        
                    if let Some(p) = dirs::document_dir() {
                        fd.config_mut().initial_directory = p;
                    }
                    fd.pick_file();
                    self.file_dialog = Some(fd);
                }
            }
        }

        // Process Selection Window
        if let Some(pd) = self.process_dialog.as_mut() {
            pd.show(ctx);
            match pd.state() {
                PDState::Closed => self.process_dialog = None,
                PDState::Selected(pid) => {
                    self.selected_process = Some(Process::new(pid));
                    self.process_dialog = None;
                }
                _ => {}
            }
        }

        // Type Selection Window
        if let Some(dialog) = self.type_selection_dialog.as_mut() {
            dialog.show(ctx);
            use type_selection_dialog::State;
            match dialog.state() {
                State::Closed => self.type_selection_dialog = None,
                State::Selected(data) => {
                    self.selected_type = Some(data.to_owned());
                    self.type_selection_dialog = None;
                }
                _ => {}
            }
        }

        /* GUI INTERFACE */
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
                if ui.button("type").clicked() {
                    self.type_selection_dialog = Some(TypeSelectionDialog::new(self.type_aliases.clone()));
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.label(format!("App state: {:?}", self.state));
            ui.add_space(10.0);

            ui.heading("File saving");
            ui.label(format!("Dirty? {}", self.is_dirty));
            ui.label(format!("File location: {:?}", self.save_file_location));
            ui.label(format!("File Dialog: {:?}", self.file_dialog.as_ref().map(|fd| fd.state())));
            ui.add_space(10.0);

            ui.heading("Process Dialog");
            ui.label(format!("Selected process: {}", self.selected_process.as_ref().and_then(|p| self.system.process(p.pid())).and_then(|p| p.name().to_str()).unwrap_or("None")));
            let pstatus = self.process_dialog.as_ref().map(|pd| pd.state());
            ui.label(format!("Process window status: {:?}", pstatus));
            ui.add_space(10.0);

            ui.heading("Type Selection Dialog");
            ui.label(format!("Selected type : {:?}", self.selected_type));
            ui.label(format!("dialog state : {:?}", self.type_selection_dialog.as_ref().map(|tsd| tsd.state())));

            if let Some(PDState::Selected(pid)) = pstatus {
                self.system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
                if let Some(p) = self.system.process(pid) {
                    
                } else {
                    self.selected_process = None
                }
            }
        });
    }
}