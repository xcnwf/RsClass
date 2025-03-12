pub mod process_dialog;
pub mod type_selection_dialog;
pub mod prompt_save_dialog;
pub mod save_dialog;
pub mod load_dialog;


#[derive(Copy, Debug, Clone, Eq, PartialEq, Default)]
pub enum DialogState<T> where T: Clone {
    #[default]
    Open,       // Visible
    Selected(T),// Selected
    Cancelled   // Cancelled
}

pub trait Dialog<T: Clone>{
    fn state(&self) -> &DialogState<T>;
    fn show(&mut self, ctx: &egui::Context);
}