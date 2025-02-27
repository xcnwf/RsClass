use eframe::egui;
use sysinfo::{System, Process, Pid, ProcessesToUpdate::All, RefreshKind, ProcessRefreshKind};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

mod typing;
use crate::typing::*;
mod gui;
use crate::gui::{ProcessDialog, State};

mod win_ops;

fn main() {
    let native_options = eframe::NativeOptions::default();
    let window_name = format!("RsClass - {}", env!("CARGO_PKG_VERSION"));
    eframe::run_native(&window_name, native_options, Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))));
}

#[derive(Default)]
struct MyEguiApp {
    element: typing::StructDataType,
    system: System,

    selected_process_id: Option<Pid>,
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
        let e = StructEntry::new("Health".into(), DataTypeEnum::Simple(Box::new(health)));
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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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
                            State::Closed | State::Selected(_) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                            _ => {}
                        }
                        if ctx.input(|i| i.viewport().close_requested()) {
                            // If we want to close, set the status as cancelled
                            match pd.state() {
                                State::Closed | State::Selected(_) => (),
                                _ => pd.cancel(),
                            }
                        }
                    }
                });
            match pd.state() {
                State::Closed => *dialog_option = None,
                State::Selected(pid) => {
                    self.selected_process_id = Some(pid);
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
                ui.button("load");
                ui.button("save");
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.label(self.element.from_bytes(&[10,0,0,0u8]).expect("WTF"));
            ui.label(format!("Selected process: {}", self.selected_process_id.and_then(|pid| self.system.process(pid)).and_then(|p| p.name().to_str()).unwrap_or("None")));
            let pstatus = self.process_dialog.lock().expect("Could not check status").as_ref().map(|pd| pd.state());
            ui.label(format!("Process window status: {:?}", pstatus));
            if let Some(State::Selected(pid)) = pstatus {
                if let Some(p) = self.system.process(pid) {
                    
                }
            }
        });
    }
}