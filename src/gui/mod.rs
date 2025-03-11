pub mod process_dialog;
pub mod type_selection_dialog;

#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub enum DialogState<T> where T: Clone {
    Open,       // Visible
    Closed,     // Destroyed
    Selected(T),// Selected
    Cancelled   // Cancelled
}

pub trait SelectDialog {
    type SelectionType: Clone;

    fn state(&self) -> &DialogState<Self::SelectionType>;
    fn show(&mut self, ctx: &egui::Context);
    fn cancel(&mut self);
    fn get_data(&self) -> Option<Self::SelectionType>;
}

pub enum Dialog {
    ProcessSelection(process_dialog::ProcessDialog),
    TypeSelection(type_selection_dialog::TypeSelectionDialog),
}