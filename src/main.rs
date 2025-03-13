use eframe::egui;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

use rs_class::{
    ops::{Process, SystemProcess},
    typing::{
        BooleanDataType, DataTypeEnum, FloatDataType, FloatPrecision, IntSize, IntegerDataType,
        StrDataType, StructDataType,
    },
};

mod gui;
use gui::load_dialog::LoadDialog;
use gui::process_dialog::ProcessDialog;
use gui::prompt_save_dialog::Choice;
use gui::save_dialog::SaveDialog;
use gui::type_selection_dialog::TypeSelectionDialog;
use gui::{Dialog, DialogState};

fn main() {
    let native_options = eframe::NativeOptions::default();
    let window_name = format!("RsClass - {}", env!("CARGO_PKG_VERSION"));
    eframe::run_native(
        &window_name,
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    )
    .expect("eframe should run");
}

// (Description: String, dt: DataTypeEnum)
type Typedef = (String, DataTypeEnum);

#[derive(Debug, Serialize, Deserialize)]
struct SaveData<'a> {
    typedefs: Cow<'a, HashMap<String, Typedef>>,
    structs: Cow<'a, Vec<StructDataType>>,
}

#[derive(Default)]
struct MyEguiApp {
    struct_tabs: Vec<StructDataType>,
    system: System,

    // type system
    typedefs: Rc<RefCell<HashMap<String, Typedef>>>,
    selected_type: Option<String>,

    selected_process: Option<Process>,
    state: AppState,

    // file saving
    save_file_location: Option<PathBuf>,
    is_dirty: bool,
}
#[derive(Debug, Clone, Copy)]
enum SaveType {
    Normal,
    Load,
    Quit,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Default)]
enum AppState {
    #[default]
    Normal,
    ProcessSelection(gui::process_dialog::ProcessDialog),
    PromptForSave(gui::prompt_save_dialog::PromptSaveDialog, SaveType),
    Load(gui::load_dialog::LoadDialog),
    Save(gui::save_dialog::SaveDialog, SaveType),
    TypeSelection(TypeSelectionDialog),
    Quit,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut s = Self::default();

        let mut typedefs = s.typedefs.borrow_mut();

        // add default datatypes
        typedefs.insert(
            String::from("Int"),
            (
                "32-bit signed integer".into(),
                IntegerDataType::default().into(),
            ),
        );
        typedefs.insert(
            String::from("UInt"),
            (
                "32-bit unsiged integer".into(),
                IntegerDataType::default().with_signed(false).into(),
            ),
        );
        typedefs.insert(
            String::from("DWORD"),
            (
                "32-bit hexadecimal integer".into(),
                IntegerDataType::default().with_hex(true).into(),
            ),
        );
        typedefs.insert(
            String::from("WORD"),
            (
                "16-bit hexadecimal integer".into(),
                IntegerDataType::default()
                    .with_hex(true)
                    .with_size(IntSize::Integer16)
                    .into(),
            ),
        );
        typedefs.insert(
            String::from("BYTE"),
            (
                "8-bit hexadecimal integer".into(),
                IntegerDataType::default()
                    .with_hex(true)
                    .with_size(IntSize::Integer8)
                    .into(),
            ),
        );
        typedefs.insert(
            String::from("Char"),
            (
                "8-bit signed integer".into(),
                IntegerDataType::default()
                    .with_size(IntSize::Integer8)
                    .into(),
            ),
        );
        typedefs.insert(
            String::from("UChar"),
            (
                "8-bit unsigned integer".into(),
                IntegerDataType::default()
                    .with_size(IntSize::Integer8)
                    .with_signed(false)
                    .into(),
            ),
        );
        typedefs.insert(
            String::from("CStr"),
            (
                "Null-ternminated string".into(),
                StrDataType::default().into(),
            ),
        );
        typedefs.insert(
            String::from("Bool"),
            (
                "Single byte boolean".into(),
                BooleanDataType::default().into(),
            ),
        );
        typedefs.insert(
            String::from("Float"),
            (
                "Simple precision floating point number".into(),
                FloatDataType::default().into(),
            ),
        );
        typedefs.insert(
            String::from("Double"),
            (
                "Double precision floating point number".into(),
                FloatDataType::default()
                    .with_precision(FloatPrecision::Double)
                    .into(),
            ),
        );
        drop(typedefs);

        let system = System::new_with_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
        );
        println!("Got {} processes.", system.processes().len());
        for (pid, p) in system.processes().iter().take(5) {
            println!("{} : {}", pid, p.name().to_str().unwrap_or_default());
        }
        s.system = system;
        s
    }

    fn save_to_file(&self) -> Result<(), String> {
        let file = std::fs::File::create(
            self.save_file_location
                .as_ref()
                .ok_or("please select a save location")?,
        )
        .map_err(|e| e.to_string())?;

        let td = self.typedefs.borrow();
        let data_to_save = SaveData {
            typedefs: Cow::Borrowed(&td),
            structs: Cow::Borrowed(&self.struct_tabs),
        };

        ron::ser::to_writer_pretty(file, &data_to_save, ron::ser::PrettyConfig::default())
            .map_err(|e| e.to_string())
    }

    fn load_from_file(&mut self) -> Result<(), String> {
        let file = std::fs::File::open(
            self.save_file_location
                .as_ref()
                .ok_or("No file path for load available")?,
        )
        .map_err(|e| e.to_string())?;
        let loaded_data: SaveData = ron::de::from_reader(file).map_err(|e| e.to_string())?;
        self.struct_tabs = loaded_data.structs.into_owned();
        self.typedefs = Rc::new(RefCell::new(loaded_data.typedefs.into_owned()));
        Ok(())
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle close requests
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested {
            match self.state {
                AppState::Save(_, _) | AppState::PromptForSave(_, _) | AppState::Load(_) => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                }
                AppState::Quit => {}
                _ => {
                    if self.is_dirty {
                        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    }
                }
            }
        }

        /* DIALOGS AND STATE TRANSITIONS */
        let nstate = match &mut self.state {
            AppState::Normal => None,
            AppState::Quit => {
                if self.is_dirty {
                    Some(AppState::PromptForSave(Default::default(), SaveType::Quit))
                } else {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    Some(AppState::Quit)
                }
            }
            AppState::Load(ld) => {
                if self.is_dirty {
                    Some(AppState::PromptForSave(Default::default(), SaveType::Load))
                } else {
                    ld.show(ctx);
                    use gui::load_dialog::State as DialogState;
                    match ld.state() {
                        DialogState::Open => None,
                        DialogState::Cancelled => Some(AppState::Normal),
                        DialogState::Selected(p) => {
                            self.save_file_location = Some(p.clone());
                            let load_success = self.load_from_file();
                            match load_success {
                                Ok(()) => {
                                    println!("Data sucessfully loaded!");
                                    self.is_dirty = false;
                                }
                                Err(err_s) => {
                                    eprintln!("ERROR: Could not load from file: {err_s}");
                                }
                            }
                            Some(AppState::Normal)
                        }
                    }
                }
            }
            AppState::Save(dialog, save_type) => {
                let st = *save_type;
                if self.save_file_location.is_some() {
                    let save_success = self.save_to_file();
                    match save_success {
                        Ok(()) => {
                            println!("File successfully saved!");
                            self.is_dirty = false;
                            match st {
                                SaveType::Quit => Some(AppState::Quit),
                                SaveType::Load => Some(AppState::Load(LoadDialog::new(self))),
                                SaveType::Normal => Some(AppState::Normal),
                            }
                        }
                        Err(err_s) => {
                            eprintln!("ERROR: Could not save to file: {err_s}");
                            Some(AppState::Normal)
                        }
                    }
                } else {
                    dialog.show(ctx);
                    match dialog.state() {
                        DialogState::Open => None,
                        DialogState::Cancelled => Some(AppState::Normal),
                        DialogState::Selected(p) => {
                            self.save_file_location = Some(p.clone());
                            None
                        }
                    }
                }
            }
            AppState::PromptForSave(dialog, save_type) => {
                dialog.show(ctx);
                match dialog.state() {
                    DialogState::Open => None,
                    DialogState::Cancelled => Some(AppState::Normal),
                    DialogState::Selected(Choice::Save) => {
                        let st = *save_type;
                        Some(AppState::Save(SaveDialog::new(self), st))
                    }
                    DialogState::Selected(Choice::Discard) => match save_type {
                        SaveType::Quit => Some(AppState::Quit),
                        SaveType::Load => Some(AppState::Load(LoadDialog::new(self))),
                        SaveType::Normal => Some(AppState::Normal),
                    },
                }
            }
            AppState::ProcessSelection(dialog) => {
                dialog.show(ctx);
                match dialog.state() {
                    DialogState::Open => None,
                    DialogState::Selected(pid) => {
                        self.selected_process = Some(Process::new(*pid));
                        Some(AppState::Normal)
                    }
                    DialogState::Cancelled => Some(AppState::Normal),
                }
            }
            AppState::TypeSelection(dialog) => {
                dialog.show(ctx);
                match dialog.state() {
                    DialogState::Open => None,
                    DialogState::Selected(data) => {
                        self.selected_type = Some(data.to_owned());
                        Some(AppState::Normal)
                    }
                    DialogState::Cancelled => Some(AppState::Normal),
                }
            }
        };

        if let Some(s) = nstate {
            self.state = s;
        }

        /* GUI INTERFACE */
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("open process").clicked() {
                    let dialog = ProcessDialog::default();
                    self.state = AppState::ProcessSelection(dialog);
                };
                if ui.button("load").clicked() {
                    self.state = AppState::Load(LoadDialog::new(self));
                };
                let save_button = egui::Button::new("save");
                if ui.add_enabled(self.is_dirty, save_button).clicked() {
                    self.state = AppState::Save(SaveDialog::new(self), SaveType::Normal);
                };
                if ui.button("type").clicked() {
                    let dialog = TypeSelectionDialog::new(self.typedefs.clone());
                    self.state = AppState::TypeSelection(dialog);
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("General Data");
            ui.label(format!("App state: {:?}", self.state));
            ui.label(format!(
                "Selected process: {}",
                self.selected_process
                    .as_ref()
                    .and_then(|p| self.system.process(p.pid()))
                    .and_then(|p| p.name().to_str())
                    .unwrap_or("None")
            ));
            ui.add_space(10.0);

            ui.heading("File saving");
            ui.label(format!("Dirty? {}", self.is_dirty));
            ui.label(format!("File location: {:?}", self.save_file_location));
            ui.add_space(10.0);

            ui.heading("Type Selection Dialog");
            ui.label(format!("Selected type : {:?}", self.selected_type));
        });
    }
}
