use egui::RichText;
use std::{rc::Rc, cell::RefCell, collections::HashMap};
use rs_class::typing::DataTypeEnum;

pub type State = super::DialogState<String>;

#[derive(Debug)]
pub struct TypeSelectionDialog {
    state: State,
    typedefs: Rc<RefCell<HashMap<String, DataTypeEnum>>>,

    search_string: String,
    selected_string: Option<String>,
}

impl TypeSelectionDialog {
    pub fn new(typedefs: Rc<RefCell<HashMap<String, DataTypeEnum>>>) -> Self {
        TypeSelectionDialog {
            state: State::Open,
            search_string: Default::default(),
            typedefs,
            selected_string: Default::default()
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

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn datatype(&self) -> Option<String> {
        match &self.state {
            State::Selected(data) => Some(data.clone()),
            _ => None
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> &Self {
        match self.state {
            State::Open => {
                let mut is_open = true;
                self.ui(ctx, &mut is_open);
                
                if is_open {
                    
                } else {
                    self.state = State::Cancelled;
                }
            },
            _ => {self.state = State::Closed},
        };

        self
    }

    fn ui(&mut self, ctx: &egui::Context, is_open: &mut bool) {
        let viewport_rect = ctx.input(|is| is.viewport().inner_rect);
        let area = egui::Modal::default_area(egui::Id::new("type_selection_dialog_area"))
            .default_size(viewport_rect.map(|r|egui::vec2(r.height()*0.75, r.width()*0.75)).unwrap_or(egui::vec2(f32::MAX, f32::MAX)));
        let window = egui::Modal::new("Process Selection Window".into()).area(area);
        if window.show(ctx, |ui| {
            self.ui_in_window(ui)
        }).should_close(){
            *is_open = false;
        }
    }

    fn ui_in_window(&mut self, ui: &mut egui::Ui) {
        ui.heading("Select a process");
        

        ui.text_edit_singleline(&mut self.search_string);

        egui::ScrollArea::vertical()
            //.max_height(viewport_rect.map(|r| r.height()/2.0).unwrap_or(f32::MAX))
            .show(ui, |ui | {
            
            for (name,_dt) in self.typedefs.borrow().iter() {
                if !name.to_lowercase().contains(&self.search_string.to_lowercase()) {
                    continue
                }
                let label_response = ui.selectable_value(
                    &mut self.selected_string,
                    Some(name.clone()), 
                    RichText::new(format!("{}", name))
                );
                if label_response.double_clicked() {
                    self.state = State::Selected(name.clone());
                }
            }
        });
        
        ui.add_space(10.0);

        egui::Sides::new().show(ui, 
            |_| {},
            |ui| {
                ui.horizontal(|ui| {
                    let ok_button = egui::Button::new("Ok");
                    if ui.add_enabled(self.selected_string.is_some(), ok_button).clicked() {
                        if let Some(data) = &self.selected_string {
                            self.state = State::Selected(data.to_owned());
                        }
                    }
                    if ui.button("Close").clicked() {
                        self.cancel();
                    }
                });
            }
        );
    }
}