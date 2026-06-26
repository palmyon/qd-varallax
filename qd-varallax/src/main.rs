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
	widgets::default_window::VxDefaultWindow,
};

fn main() {
	let mut app = VxApplication::new();
	let window = VxDefaultWindow::new(Default::default());
	app.add_window(window);
	app.exec();
}