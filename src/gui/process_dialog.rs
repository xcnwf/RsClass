/***
 * Process selection dialog
 * Inspired by egui_file
 * 
*/

use std::{cell::RefCell, collections::HashSet, rc::Rc};

use egui::RichText;
use sysinfo::{Pid, System, ProcessesToUpdate::All};

/*
 * From ReClassEx : https://github.com/ajkhoury/ReClassEx/blob/master/ReClass/DialogProcSelect.cpp
 */
lazy_static::lazy_static! {
    static ref common_processes : HashSet<&'static str> = 
    [
        "svchost.exe", "System", "conhost.exe", "wininit.exe", "smss.exe", "winint.exe", "wlanext.exe", "Code.exe", "taskhostw.exe", "SearchIndexer.exe", "Idle", "SearchApp.exe", "dllhost.exe",
        "spoolsv.exe", "spoolsv.exe", "notepad.exe", "explorer.exe", "itunes.exe",
        "sqlservr.exe", "nvtray.exe","nvxdsync.exe", "lsass.exe", "jusched.exe",
        "conhost.exe", "chrome.exe", "firefox.exe", "winamp.exe", "TrustedInstaller.exe",
        "WinRAR.exe", "calc.exe", "taskhostex.exe", "Taskmgr.exe","dwm.exe","SpotifyWebHelper.exe",
        "plugin-container.exe", "services.exe", "devenv.exe", "flux.exe", "skype.exe", "spotify.exe",
        "csrss.exe", "taskeng.exe", "spotifyhelper.exe", "vcpkgsrv.exe", "msbuild.exe", "cmd.exe", "taskhost.exe",
        "SettingSyncHost.exe", "SkyDrive.exe", "ctfmon.exe", "RuntimeBroker.exe", "BTTray.exe", "BTStackServer.exe",
        "Bluetooth Headset Helper.exe", "winlogon.exe", "PnkBstrA.exe", "armsvc.exe", "MSIAfterburner.exe", "vmnat.exe",
        "vmware-authd.exe", "vmnetdhcp.exe", "pia_manager.exe", "SpotifyWebHelper.exe", "Dropbox.exe", "Viber.exe", "idaq.exe",
        "idaq64.exe", "CoreSync.exe", "SpotifyCrashService.exe", "RzSynapse.exe", "acrotray.exe",
        "CCLibrary.exe", "pia_tray.exe", "rubyw.exe", "netsession_win.exe", "NvBackend.exe", "TeamViewer_Service.exe",
        "DisplayFusionHookAppWIN6032.exe", "DisplayFusionHookAppWIN6064.exe", "GameScannerService.exe", "AdobeUpdateService.exe",
        "steamwebhelper.exe", "c2c_service.exe", "Sync Server.exe", "NvNetworkService.exe", "Creative Cloud.exe", "foobar2000.exe",
        "code.exe", "ReClass.exe", "ReClass64.exe", "Discord.exe", "node.exe", "TeamViewer.exe", "Everything.exe"
    ].into();
}

use super::DialogState;
pub type State = DialogState<Pid>;

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
        let viewport_rect = ui.input(|is| is.viewport().inner_rect);

        egui::ScrollArea::vertical()
            .max_height(viewport_rect.map(|r| r.height()/2.0).unwrap_or(f32::MAX))
            .show(ui, |ui | {
            
            for (pid, process) in self.system.processes() {
                if common_processes.contains(&process.name().to_str().unwrap_or_default()) {
                    continue
                }
                let label_response = ui.selectable_value(
                    &mut self.selected_process_id,
                    Some(*pid), 
                    RichText::new(format!("{:>6} | {}", pid.as_u32(), process.name().to_str().unwrap_or_default())).monospace()
                );
                if label_response.double_clicked() {
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