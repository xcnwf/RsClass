use relm4::prelude::*;

mod gui;
mod typing;

fn main() {
    println!("Hello, world!");
    let relm = RelmApp::new("test");
    relm.run::<gui::App>(());
}
