#![allow(dead_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod abstractions;
mod core;
mod types;
mod utils;
mod painter;
mod widgets;
mod develop_examples;

use crate::{
	core::application::VxApplication,
	develop_examples::main_window::MainWindow,
};

fn main() {
    let mut app = VxApplication::new();
	let window = MainWindow::new(Default::default());
	app.add_window(window);
	app.exec();
}