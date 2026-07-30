use std::{sync::Arc, time::{Duration, Instant}};

use ahash::AHashMap;
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
	window::{
		Window,
		WindowId
	}
};

use crate::{
	abstractions::abstract_windows::{
		VxWindow, VxWindowStats
	}, core::resource::VxAppResource, types::{
		event::{
			VxEvent,
			VxKeyEvent,
			VxMouseEvent,
			VxWindowEvent
		},
		geometry::{
			VxSize,
			VxVec2
		}
	}
};


struct VxAppHandler {
	resources: Option<VxAppResource>,
	windows: AHashMap<WindowId, (Arc<Window>, Box<dyn VxWindow>)>,
	init_windows: Vec<Box<dyn VxWindow>>,
	last_frame_time: Instant,
	target_frame_duration: Duration,
	last_mouse_pos: VxVec2,
	wheel_pixel_amount: f32,
	proxy: EventLoopProxy<VxEvent>,
}

impl ApplicationHandler<VxEvent> for VxAppHandler {
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		if self.resources.is_some() {
			return;
		}
		for window in std::mem::take(&mut self.init_windows) {
			let Some((winit_window, w)) = Self::create_window(event_loop, window) else { continue; };
			let id = winit_window.id();
			self.windows.insert(id, (Arc::new(winit_window), w));
		}

		let res = self.resources.get_or_insert_with(|| VxAppResource::new());

		for (winit_window, window) in self.windows.values_mut() {
			if window.stats().is_none() {
				let stats = VxWindowStats::new(&res.gpu, winit_window.clone(), self.proxy.clone());
				window.set_stats(stats);
				window.init_event();
				window.stats_mut().as_mut().map(|s| s.finalize_init());
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
		let Some((_, window)) = self.windows.get_mut(&window_id) else { return; };

		match event {
			WindowEvent::RedrawRequested => {
				window.update_event(resource);
			}
			WindowEvent::Resized(size) => {
				let vx_event = VxWindowEvent::new(
					VxSize::from_u32(size.width, size.height)
				);
				window.handle_event(&resource.gpu, &VxEvent::ResizeEvent { event: vx_event });
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
					window.handle_event(&resource.gpu, &VxEvent::MousePressEvent { event: vx_event });
				} else {
					let vx_event = VxMouseEvent::new(
						self.last_mouse_pos,
						Some(button),
						VxVec2::default(),
					);
					window.handle_event(&resource.gpu, &VxEvent::MouseReleaseEvent { event: vx_event });
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
				window.handle_event(&resource.gpu, &VxEvent::MouseMoveEvent { event: vx_event });
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
				window.handle_event(&resource.gpu, &VxEvent::MouseWheelEvent { event: vx_event });
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
					window.handle_event(&resource.gpu, &VxEvent::KeyPressedEvent { event: vx_event });
				} else {
					let vx_event = VxKeyEvent::new(code, event.state.is_pressed());
					window.handle_event(&resource.gpu, &VxEvent::KeyReleasedEvent { event: vx_event });
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

				let Some((winit_window, mut w)) = Self::create_window(event_loop, window) else { return; };
				let id = winit_window.id();
				let arc_window = Arc::new(winit_window);
				w.set_stats(VxWindowStats::new(&res.gpu, arc_window.clone(), self.proxy.clone()));
				w.init_event();
				w.stats_mut().as_mut().map(|s| s.finalize_init());
				self.windows.insert(id, (arc_window, w));
			}
			_ => {}
		}
	}
	fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		let now = Instant::now();
		let elapsed = now.duration_since(self.last_frame_time);
		if elapsed >= self.target_frame_duration {
			for (_, window) in self.windows.values_mut() {
				let has_immediate = window.has_immediate();
				if let (Some(res), Some(stats)) = (&mut self.resources, window.stats_mut()) {
					if has_immediate {
						if !stats.check_dirty(res) {
							stats.window.request_redraw();
						}
					} else {
						stats.check_dirty(res);
					}
				}
			}
			self.last_frame_time = now;
			event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
		} else {
			let timeout = self.target_frame_duration - elapsed;
			event_loop.set_control_flow(winit::event_loop::ControlFlow::wait_duration(timeout));
		}
	}
}

impl VxAppHandler {
	pub(crate) fn create_window(
		event_loop: &winit::event_loop::ActiveEventLoop,
		window: Box<dyn VxWindow>,
	) -> Option<(Window, Box<dyn VxWindow>)> {
		let attr = window.create_window_attr();
		let winit_window = event_loop.create_window(attr).ok()?;
		Some((winit_window, window))
	}
}

pub struct VxApplication {
	event_loop: EventLoop<VxEvent>,
	handler: VxAppHandler,
}

impl VxApplication {
	pub fn new() -> Self {
		let mut builder = EventLoop::<VxEvent>::with_user_event();
		let event_loop = builder.build()
			.expect("VxApplication> new(): Failed to Create EventLoop");
		event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);

		let proxy = event_loop.create_proxy();

		Self {
			event_loop,
			handler: VxAppHandler {
				resources: None,
				windows: AHashMap::new(),
				init_windows: Vec::new(),
				last_frame_time: Instant::now(),
				target_frame_duration: Duration::from_secs_f64(1.0 / 60.0),
				last_mouse_pos: VxVec2::default(),
				wheel_pixel_amount: 15.0,
				proxy
			}
		}
	}
	#[inline]
	pub fn with_target_frame_rate(mut self, target_frame_rate: f64) -> Self {
		self.handler.target_frame_duration = Duration::from_secs_f64(1.0 / target_frame_rate);
		self
	}
	#[inline]
	pub fn with_target_frame_duration_ms(mut self, target_frame_rate_ms: u64) -> Self {
		self.handler.target_frame_duration = Duration::from_mins(target_frame_rate_ms);
		self
	}
	#[inline]
	pub fn add_window<W: VxWindow + 'static>(&mut self, window: W) {
		self.handler.init_windows.push(Box::new(window));
	}
	#[inline]
	pub fn exec(mut self) {
		self.event_loop.run_app(&mut self.handler).unwrap();
	}
}