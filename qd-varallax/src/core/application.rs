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
	}, core::resource::VxAppResource, types::{event::{VxEvent, VxKeyEvent, VxMouseEvent, VxWindowEvent}, geometry::{
			VxSize,
			VxVec2
		}}
};


struct VxAppHandler {
	resources: Option<VxAppResource>,
	windows: HashMap<WindowId, Box<dyn VxWindow>>,
	init_windows: Vec<Box<dyn VxWindow>>,
	last_mouse_pos: VxVec2,
	wheel_pixel_amount: f32,
	proxy: EventLoopProxy<VxEvent>,
}

impl ApplicationHandler<VxEvent> for VxAppHandler {
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		// 初期化前なら初期化
		if self.resources.is_none() {
			self.resources = Some(VxAppResource::new());
		} else {
			return;
		}

		let Some(resource) = &self.resources else { return; };
		
		for window in std::mem::take(&mut self.init_windows) {
			let Some((id, w)) = Self::create_window(event_loop, resource, window, self.proxy.clone()) else { continue; };
			self.windows.insert(id, w);
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
				let window = builder.build();

				let Some(res) = &self.resources else {
					self.init_windows.push(window);
					return;
				};

				let Some((id, w)) = Self::create_window(event_loop, res, window, self.proxy.clone()) else { return; };
				self.windows.insert(id, w);
			}
			_ => {}
		}
	}
	fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		let mut has_immediate = false;
		for window in self.windows.values_mut() {
			if window.has_immediate() {
				has_immediate = true;
				if let Some(stat) = window.stats_mut() {
					stat.set_dirty(true);
					stat.check_dirty();
				}
			} else {
				if let Some(stat) = window.stats_mut() {
					stat.check_dirty();
				}
			}
		}
		let flow = if has_immediate { winit::event_loop::ControlFlow::Poll } else { winit::event_loop::ControlFlow::Wait };
		event_loop.set_control_flow(flow);
	}
}

impl VxAppHandler {
	pub(crate) fn create_window(
		event_loop: &winit::event_loop::ActiveEventLoop,
		res: &VxAppResource,
		mut window: Box<dyn VxWindow>,
		proxy: EventLoopProxy<VxEvent>,
	) -> Option<(WindowId, Box<dyn VxWindow>)> {
		let attr = window.create_window_attr();
		let winit_window = event_loop.create_window(attr).ok()?;
		let id = winit_window.id();
		let stats = VxWindowStats::new(&res.gpu, winit_window, proxy);
		window.set_stats(stats);
		window.init_event();
		if let Some(s) = window.stats_mut() {
			s.finalize_init();
		}
		Some((id, window))
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