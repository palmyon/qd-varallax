use winit::{
	dpi::LogicalSize,
	event_loop::EventLoopProxy,
	window::{
		Window,
		WindowButtons
	}
};

use crate::{
	abstractions::abstract_windows::{VxWindowBuilder, VxWindowLayer}
	,
	core::vx_event::VxEvent,
	types::geometry::VxSize
};

pub(crate) struct VxWindowFunctions;

impl VxWindowFunctions {
	// サイズ関係
	pub fn set_fixed_size(window: &Window, size: Option<VxSize>) {
		if let Some(s) = size {
			let logic_size = LogicalSize::new(
				s.width(),
				s.height(),
			);
			let _ = window.request_inner_size(logic_size);
			Self::set_window_resizable(&window, false);
		} else {
			Self::set_window_resizable(&window, true);
		}
	}

	pub fn set_minimum_size(window: &Window, size: Option<VxSize>) {
		if let Some(s) = size {
			let logic_size = LogicalSize::new(
				s.width(),
				s.height(),
			);
			window.set_min_inner_size(Some(logic_size));
		} else {
			window.set_min_inner_size::<LogicalSize<f32>>(None);
		}
	}

	pub fn set_maximum_size(window: &Window, size: Option<VxSize>) {
		if let Some(s) = size {
			let logic_size = LogicalSize::new(
				s.width(),
				s.height(),
			);
			window.set_max_inner_size(Some(logic_size));
		} else {
			window.set_max_inner_size::<LogicalSize<f32>>(None);
		}
	}

	pub fn set_window_resizable(window: &Window, resizable: bool) {
		window.set_resizable(resizable);
		if resizable {
			window.set_enabled_buttons(WindowButtons::all());
		} else {
			window.set_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE);
		}
	}

	pub fn set_transparent(window: &Window, transparent: bool) {
		window.set_transparent(transparent);
	}

	pub fn show_fullscreen(window: &Window) {
		if window.fullscreen().is_none() {
			window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
		}
	}

	pub fn show_normal(window: &Window) {
		if window.fullscreen().is_some() {
			window.set_fullscreen(None);
		}
	}

	pub fn is_fullscreen(window: &Window) -> bool {
		window.fullscreen().is_some()
	}

	pub fn close(window: &Window, proxy: &EventLoopProxy<VxEvent>) {
		let _ = proxy.send_event(VxEvent::CloseEvent { window_id: window.id() });
	}

	pub fn set_window_layer(window: &Window, layer: VxWindowLayer) {
		window.set_window_level(layer.into());
	}

	pub fn set_window_minimizable(window: &Window, minimizable: bool) {
		let mut buttons = window.enabled_buttons();
		buttons.set(WindowButtons::MINIMIZE, minimizable);
		window.set_enabled_buttons(buttons);
	}

	pub fn show(builder: Box<dyn VxWindowBuilder>, proxy: &EventLoopProxy<VxEvent>) {
		let _ = proxy.send_event(VxEvent::ShowEvent { builder });
	}
}