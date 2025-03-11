pub mod process_dialog;
pub mod type_selection_dialog;

#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub enum DialogState<T> where T: Clone {
    Open,       // Visible
    Selected(T),// Selected
    Cancelled   // Cancelled
}

pub trait Dialog<T: Clone>{
    fn state(&self) -> &DialogState<T>;
    fn show(&mut self, ctx: &egui::Context);
    fn cancel(&mut self);
    fn get_data(&self) -> Option<&T>;
}