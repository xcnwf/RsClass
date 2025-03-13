use std::{path::PathBuf, sync::Arc};

use rs_class::ops::SystemProcess;

pub type State = super::DialogState<PathBuf>;

#[derive(Debug)]
pub struct SaveDialog {
    state: State,
    file_dialog: egui_file_dialog::FileDialog,
}

impl SaveDialog {
    pub fn new(app: &crate::MyEguiApp) -> Self {
        let mut fd: egui_file_dialog::FileDialog = egui_file_dialog::FileDialog::new()
            .add_file_filter(
                "rsclass",
                Arc::new(|path| path.extension().is_some_and(|ext| ext == "rsclass")),
            )
            .default_file_filter("rsclass");
        if let Some(s) = app
            .selected_process
            .as_ref()
            .and_then(|p| app.system.process(p.pid()))
            .and_then(|p| p.name().to_str())
            .map(PathBuf::from)
            .and_then(|path| {
                path.with_extension("rsclass")
                    .to_str()
                    .map(ToOwned::to_owned)
            })
        {
            fd.config_mut().default_file_name = s;
        }

        if let Some(p) = dirs::document_dir() {
            fd.config_mut().initial_directory = p;
        }

        fd.save_file();

        Self {
            state: Default::default(),
            file_dialog: fd,
        }
    }
}

impl super::Dialog<PathBuf> for SaveDialog {
    fn show(&mut self, ctx: &egui::Context) {
        self.file_dialog.update(ctx);
        use egui_file_dialog::DialogState::{Cancelled, Open, Picked};
        match self.file_dialog.state() {
            Open => {}
            Picked(p) => self.state = State::Selected(p.clone()),
            Cancelled => {
                println!("User did not choose save file, saving is cancelled");
                self.state = State::Cancelled;
            }
            _ => unreachable!(),
        }
    }

    fn state(&self) -> &super::DialogState<PathBuf> {
        &self.state
    }
}
