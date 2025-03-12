#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Choice {
    Save,
    Discard
}
pub type State = super::DialogState<Choice>;

#[derive(Debug, Default)]
pub struct PromptSaveDialog {
    state: State,
}

impl super::Dialog<Choice> for PromptSaveDialog {
    fn show(&mut self, ctx: &egui::Context) {
        egui::Modal::new("close_unsaved_dialog".into()).show(ctx, |ui| {
            ui.heading("Do you wish to save your changes ?");
            ui.horizontal(|ui| {
                if ui.button("Discard changes").clicked() {
                    self.state = State::Selected(Choice::Discard);
                }
                else if ui.button("Save changes").clicked() {
                    self.state = State::Selected(Choice::Save);
                }
                else if ui.button("Cancel").clicked() {
                    self.state = State::Cancelled;
                }
            })
        });
    }

    fn cancel(&mut self) {
        self.state = State::Cancelled;
    }

    fn get_data(&self) -> Option<&Choice> {
        if let State::Selected(c) = &self.state {
            Some(c)
        } else {
            None
        }
    }
    fn state(&self) -> &super::DialogState<Choice> {
        &self.state
    }
}