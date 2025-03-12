use std::{path::PathBuf, sync::Arc};

use rs_class::ops::SystemProcess;

pub type State = super::DialogState<PathBuf>;

#[derive(Debug)]
pub struct LoadDialog {
    state: State,
    file_dialog: egui_file_dialog::FileDialog,
}

impl LoadDialog {
    pub fn new(app: &crate::MyEguiApp) -> Self {
        let mut fd: egui_file_dialog::FileDialog = egui_file_dialog::FileDialog::new()
            .add_file_filter("rsclass", Arc::new(|path| path.extension().map(|ext| ext == "rsclass").unwrap_or_default()))
            .default_file_filter("rsclass");
        
        if let Some(s) = app
            .selected_process
            .as_ref()
            .and_then(|p| app.system.process(p.pid()))
            .and_then(|p| p.name().to_str())
            .map(|s| PathBuf::from(s))
            .and_then(|path| path.with_extension("rsclass").to_str().map(ToOwned::to_owned))
        {
            fd.config_mut().default_file_name = s;
        }

        if let Some(p) = dirs::document_dir() {
            fd.config_mut().initial_directory = p;
        }

        fd.pick_file();

        Self {
            state: Default::default(),
            file_dialog: fd,
        }
    }
}

impl super::Dialog<PathBuf> for LoadDialog{
    fn show(&mut self, ctx: &egui::Context) {
        self.file_dialog.update(ctx);
        use egui_file_dialog::DialogState::*;
        match self.file_dialog.state() {
            Open => {},
            Picked(p) => self.state = State::Selected(p.clone()),
            Cancelled => {
                self.state = State::Cancelled;
                println!("User did not choose save file, saving is cancelled");
            },
            _ => unreachable!(),
        }
    }

    fn cancel(&mut self) {
        self.state = State::Cancelled;
    }

    fn get_data(&self) -> Option<&PathBuf> {
        if let State::Selected(c) = &self.state {
            Some(c)
        } else {
            None
        }
    }
    fn state(&self) -> &super::DialogState<PathBuf> {
        &self.state
    }
}