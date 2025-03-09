pub mod process_dialog;
pub mod type_selection_dialog;

#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub enum DialogState<T> {
    Open,       // Visible
    Closed,     // Destroyed
    Selected(T),// Selected
    Cancelled   // Cancelled
}