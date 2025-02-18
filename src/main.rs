use eframe::egui;
use sysinfo::{System, Process, Pid, ProcessesToUpdate::All, RefreshKind, ProcessRefreshKind};
use std::sync::{Arc, Mutex};

//mod gui;
mod typing;
use crate::typing::*;
mod gui;
use crate::gui::{ProcessDialog, State};

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
    process_dialog: Option<ProcessDialog>,
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
        if let Some(pd) = &mut self.process_dialog {
            // egui::Modal::new(egui::Id::from("Selection Window"))
            //     .max_height(ctx.input(|is| is.viewport().inner_rect.map(|r| r.height()).unwrap_or(100.0)))
            //     .show(ctx, |ui| {
            //         ui.heading("Select an option");
            //         egui::ScrollArea::vertical().show(ui, |ui | {
            //             let spid = self.selected_process.unwrap_or(Pid::from_u32(0));
            //             for (pid, process) in self.system.processes() {
            //                 let label_response = ui.selectable_label(
            //                     *pid == spid, 
            //                     format!("{:6} | {}", pid, process.name().to_str().unwrap_or_default())
            //                 );
            //                 if label_response.clicked() {
            //                     self.selected_process = Some(*pid)
            //                 }
            //                 if label_response.double_clicked() {
            //                     self.opened_process_id = Some(pid.to_owned());
            //                     self.selected_process = None;
            //                     self.show_process_selection_window = false;
            //                 }
            //             }
            //         });
                    
            //         if ui.button("Refresh").clicked() {
            //             self.system.refresh_processes(All, true);
            //         }
            //         egui::Grid::new("selection_grid").show(ui, |ui| {
            //             let ok_button = egui::Button::new("Ok");
            //             if ui.add_enabled(self.selected_process.is_some(), ok_button).clicked() {
            //                 self.opened_process_id = self.selected_process;
            //                 self.selected_process = None;
            //                 self.show_process_selection_window = false;
            //             } 
            //             if ui.button("Close").clicked() {
            //                 self.selected_process = None;
            //                 self.show_process_selection_window = false;
            //             }
            //         }) 
            //     });

            // ctx.show_viewport_deferred(
            //     egui::ViewportId::from_hash_of("Process selection"),
            //     egui::ViewportBuilder::default(), 
            //      |ctx, _vp_class| {
            //         apd_clone.lock().expect("Could not display process selection window").show(ctx);
            //     });
            pd.show(ctx);
            // let pd = apd.lock().expect("Could not get process selection result");
            match pd.state() {
                State::Closed => self.process_dialog = None,
                State::Selected => {
                    self.selected_process_id = pd.pid();
                    self.process_dialog = None
                }
                _ => {}
            }
        }
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("open process").clicked() {
                    let mut dialog = ProcessDialog::new();
                    dialog.open();
                    self.process_dialog = Some(dialog);
                };
                ui.button("load");
                ui.button("save");
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.label(self.element.from_bytes(&[10,0,0,0u8]).expect("WTF"));
            ui.label(format!("Selected process: {}", self.selected_process_id.and_then(|pid| self.system.process(pid)).and_then(|p| p.name().to_str()).unwrap_or("None")))
        });
    }
}