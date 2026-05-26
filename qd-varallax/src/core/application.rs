use std::collections::HashMap;

use winit::{
	application::ApplicationHandler,
	event::{
		MouseScrollDelta,
		WindowEvent
	},
	event_loop::{
		EventLoop,
		EventLoopProxy
	},
	keyboard::{
		KeyCode,
		PhysicalKey
	},
	window::WindowId
};

use crate::{
	abstractions::abstract_windows::{
		VxWindow,
		VxWindowStats
	},
	core::{
		resource::VxAppResources, vx_event::{
			VxEvent,
			VxKeyEvent,
			VxMouseEvent,
			VxWindowEvent
		}
	},
	types::geometry::{
			VxSize,
			VxVec2
		}
};


struct VxAppHandler {
	resources: Option<VxAppResources>,
	windows: HashMap<WindowId, Box<dyn VxWindow>>,
	init_windows: Vec<Box<dyn VxWindow>>,

	last_mouse_pos: VxVec2,
	wheel_pixel_amount: f32,
	proxy: EventLoopProxy<VxEvent>
}

impl ApplicationHandler<VxEvent> for VxAppHandler {
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		// 初期化前なら初期化
		if self.resources.is_none() {
			self.resources = Some(VxAppResources::new());
		}

		let Some(resource) = &self.resources else { return; };
		
		for mut wndw in self.init_windows.drain(..) {
			let attr = wndw.create_window_attr();
			if let Ok(window) = event_loop.create_window(attr) {
				let id = window.id();
				let stats = VxWindowStats::new(&resource.gpu, window, self.proxy.clone());
				wndw.set_stats(stats);
				wndw.init_event();
				if let Some(s) = wndw.stats_mut() {
					s.finalize_init();
				}

				self.windows.insert(id, wndw);
				}
			}
		}

	fn window_event(
			&mut self,
			event_loop: &winit::event_loop::ActiveEventLoop,
			window_id: WindowId,
			event: winit::event::WindowEvent,
		) {
		let Some(resource) = &mut self.resources else { return; };
		let Some(window) = self.windows.get_mut(&window_id) else { return; };

		match event {
			WindowEvent::RedrawRequested => {
				window.update_event(resource);
			}
			WindowEvent::Resized(size) => {
				window.chain_resize_event(&resource.gpu, size);
				let vx_event = VxWindowEvent::new(
					VxSize::from_u32(size.width, size.height)
				);
				window.handle_event(&VxEvent::ResizeEvent { event: vx_event });
			},
			WindowEvent::CloseRequested => {
				self.windows.remove(&window_id);
				if self.windows.is_empty() {
					event_loop.exit();
				}
			},

			WindowEvent::MouseInput { state, button, .. } => {
				if state.is_pressed() {
					let vx_event = VxMouseEvent::new(
						self.last_mouse_pos,
						Some(button),
						VxVec2::default(),
					);
					window.handle_event(&VxEvent::MousePressEvent { event: vx_event });
				} else {
					let vx_event = VxMouseEvent::new(
						self.last_mouse_pos,
						Some(button),
						VxVec2::default(),
					);
					window.handle_event(&VxEvent::MouseReleaseEvent { event: vx_event });
				}
			}

			WindowEvent::CursorMoved { position, .. } => {
				let pos = VxVec2::new(position.x as f32, position.y as f32);
				if let Some(stat) = window.stats() {
					let factor = stat.scale_factor();
					self.last_mouse_pos = pos / factor;
				}

				let vx_event = VxMouseEvent::new(
					self.last_mouse_pos,
					None,
					VxVec2::default(),
				);
				window.handle_event(&VxEvent::MouseMoveEvent { event: vx_event });
			}

			WindowEvent::MouseWheel { delta, .. } => {
				let wheel_delta = {
					match delta {
						MouseScrollDelta::LineDelta(x, y) => {
							VxVec2::new(
								x * self.wheel_pixel_amount,
								y * self.wheel_pixel_amount
							)
						},
						MouseScrollDelta::PixelDelta(pos) => {
							VxVec2::new(pos.x as f32, pos.y as f32)
						}
					}
				};
				let vx_event = VxMouseEvent::new(
					self.last_mouse_pos,
					None,
					wheel_delta
				);
				window.handle_event(&VxEvent::MouseWheelEvent { event: vx_event });
			}

			WindowEvent::KeyboardInput { event, .. } => {
				let key = event.physical_key;
				let code = {
					match key {
						PhysicalKey::Code(key_code) => {
							key_code
						},
						PhysicalKey::Unidentified(..) => {
							KeyCode::KeyA
						}
					}
				};
				if event.state.is_pressed() {
					let vx_event = VxKeyEvent::new(code, event.state.is_pressed());
					window.handle_event(&VxEvent::KeyPressedEvent { event: vx_event });
				} else {
					let vx_event = VxKeyEvent::new(code, event.state.is_pressed());
					window.handle_event(&VxEvent::KeyReleasedEvent { event: vx_event });
				}
			}
			_ => {},
		}
	}
	fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: VxEvent) {
		match event {
			VxEvent::CloseEvent { window_id } => {
				self.windows.remove(&window_id);
				if self.windows.is_empty() {
					event_loop.exit();
				}
			},
			VxEvent::ShowEvent { builder } => {
				let mut wndw = builder.build();

				if let Ok(window) = event_loop.create_window(wndw.create_window_attr()) {
					let Some(resources) = &self.resources else { return; };
					let id = window.id();
					let stats = VxWindowStats::new(&resources.gpu, window, self.proxy.clone());
					wndw.set_stats(stats);
					wndw.init_event();
					if let Some(s) = wndw.stats_mut() {
						s.finalize_init();
					}

					self.windows.insert(id, wndw);
				}
			}
			_ => {}
		}
	}
	fn about_to_wait(&mut self, _: &winit::event_loop::ActiveEventLoop) {
		for window in self.windows.values_mut() {
			if let Some(stat) = window.stats_mut() {
				stat.check_dirty();
			}
		}
	}
}

pub struct VxApplication {
	event_loop: EventLoop<VxEvent>,
	handler: VxAppHandler,
}

impl VxApplication {
	pub fn new() -> Self {
		let mut builder = EventLoop::<VxEvent>::with_user_event();
		let event_loop = builder.build().expect("VxApplication> new(): Failed to Create EventLoop");
		event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

		let proxy = event_loop.create_proxy();

		Self {
			event_loop,
			handler: VxAppHandler {
				resources: None,
				windows: HashMap::new(),
				init_windows: Vec::new(),
				last_mouse_pos: VxVec2::default(),
				wheel_pixel_amount: 15.0,
				proxy
			}
		}
	}

	pub fn add_window<W: VxWindow + 'static>(&mut self, window: W) {
		self.handler.init_windows.push(Box::new(window));
	}

	pub fn exec(mut self) {
		self.event_loop.run_app(&mut self.handler).unwrap();
	}
}