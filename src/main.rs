use eframe::egui;
use sysinfo::{System, RefreshKind, ProcessRefreshKind};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

use rs_class::{typing::*, win_ops::*};

mod gui;
use gui::{ProcessDialog, State as GuiState};

fn main() {
    let native_options = eframe::NativeOptions::default();
    let window_name = format!("RsClass - {}", env!("CARGO_PKG_VERSION"));
    eframe::run_native(&window_name, native_options, Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc))))).expect("eframe should run");
}

#[derive(Default)]
struct MyEguiApp {
    element: StructDataType,
    system: System,

    selected_process: Option<WinProcess>,
    process_dialog: Arc<Mutex<Option<ProcessDialog>>>,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let mut s = Self::default();
        let health = IntegerDataType::default();
        let e = StructEntry::new("Health".into(), health.into());
        s.element.push_entry(e);

        let system = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()));
        println!("Got {} processes.", system.processes().len());
        for (pid, p) in system.processes().iter().take(5) {
            println!("{} : {}", pid, p.name().to_str().unwrap_or_default());
        }
        s.system = system;
        s
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut dialog_option = self.process_dialog.lock().expect("Could not update process dialog");
        if let Some(pd) = dialog_option.deref_mut() {
            let apd = self.process_dialog.clone();
            ctx.show_viewport_deferred(
                egui::ViewportId::from_hash_of("Process selection"),
                egui::ViewportBuilder::default(), 
                 move |ctx, _class| {
                    if let Some(pd) = apd.lock().expect("Could not display process selection window").deref_mut() {
                        pd.show(ctx);
                        match pd.state() {
                            GuiState::Closed | GuiState::Selected(_) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                            _ => {}
                        }
                        if ctx.input(|i| i.viewport().close_requested()) {
                            // If we want to close, set the status as cancelled
                            match pd.state() {
                                GuiState::Closed | GuiState::Selected(_) => (),
                                _ => pd.cancel(),
                            }
                        }
                    }
                });
            match pd.state() {
                GuiState::Closed => *dialog_option = None,
                GuiState::Selected(pid) => {
                    self.selected_process = Some(WinProcess::new(pid));
                    *dialog_option = None;
                }
                _ => {}
            }
        }
        drop(dialog_option);
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("open process").clicked() {
                    let mut dialog = ProcessDialog::new();
                    dialog.open();
                    self.process_dialog = Arc::new(Mutex::new(Some(dialog)));
                };
                if ui.button("load").clicked() {};
                if ui.button("save").clicked() {};
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.label(self.element.from_bytes(&[10,0,0,0u8]).expect("WTF"));
            ui.label(format!("Selected process: {}", self.selected_process.as_ref().and_then(|p| self.system.process(p.pid())).and_then(|p| p.name().to_str()).unwrap_or("None")));
            let pstatus = self.process_dialog.lock().expect("Could not check status").as_ref().map(|pd| pd.state());
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