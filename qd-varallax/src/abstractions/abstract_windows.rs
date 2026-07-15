use std::sync::Arc;

use winit::{
	dpi::LogicalSize,
	event_loop::EventLoopProxy,
	window::{Window, WindowAttributes, WindowLevel},
};

use crate::{
	abstractions::{
		abstract_widgets::{
			VxWidget, 
			VxWidgetHandler
		},
		window_function::VxWindowFunctions
	},
	core::{
		gpu_resource::VxGpuResource,
		renderer::VxRenderer,
		resource::VxAppResource,
		scene::VxScene,
	},
	painter::painter::VxPainter, 
	types::{
		event::{
			VxEvent,
			VxEventResult,
			VxKeyEvent,
			VxMouseEvent,
			VxWindowEvent
		},
		geometry::VxSize,
		input::{
			VxInputState,
			VxKeyModifierState,
			VxKeyboardState,
			VxMouseState
		},
		render_commands::{
			VxDirtyCheckResult,
			VxRenderMode
		},
		transform::VxMatrix4x4
	},
};

#[derive(Clone, Debug, PartialEq)]
pub struct VxWindowAttributes {
	title: String,
	size: VxSize,
}

impl Default for VxWindowAttributes {
	fn default() -> Self {
		Self {
			title: "VxWindow".into(),
			size: VxSize::from_i32(1280, 720),
		}
	}
}

impl VxWindowAttributes {
	pub fn new(title: impl Into<String>, window_size: VxSize) -> Self {
		Self {
			title: title.into(),
			size: window_size,
		}
	}
	pub(crate) fn create_window_attr(&self) -> WindowAttributes {
		let attr = WindowAttributes::default()
			.with_title(&self.title)
			.with_inner_size(LogicalSize::new(self.size.width(), self.size.height()));
		attr
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VxWindowLayer {
	AlwaysOnTopLayer,
	AlwaysOnBottomLayer,
	NormalLayer,
}

impl From<WindowLevel> for VxWindowLayer {
	fn from(level: WindowLevel) -> Self {
		match level {
			WindowLevel::AlwaysOnBottom => VxWindowLayer::AlwaysOnBottomLayer,
			WindowLevel::AlwaysOnTop => VxWindowLayer::AlwaysOnTopLayer,
			WindowLevel::Normal => VxWindowLayer::NormalLayer,
		}
	}
}

impl From<VxWindowLayer> for WindowLevel {
	fn from(layer: VxWindowLayer) -> Self {
		match layer {
			VxWindowLayer::AlwaysOnBottomLayer => WindowLevel::AlwaysOnBottom,
			VxWindowLayer::AlwaysOnTopLayer => WindowLevel::AlwaysOnTop,
			VxWindowLayer::NormalLayer => WindowLevel::Normal,
		}
	}
}

pub trait VxWindowBuilder: Send + 'static {
	fn build(self: Box<Self>) -> Box<dyn VxWindow>;
	fn window_attr_b(&self) -> &VxWindowAttributes;
}

pub struct VxWindowStats {
	pub(crate) window: Arc<Window>,
	pub(crate) proxy: EventLoopProxy<VxEvent>,
	surface: wgpu::Surface<'static>,
	config: wgpu::SurfaceConfiguration,
	renderer: VxRenderer,
	scene: VxScene,
	input: VxInputState,

	next_update_mode: VxDirtyCheckResult,
	is_dirty: bool,
}

impl VxWindowStats {
	pub fn new(gpu: &VxGpuResource, window: Window, proxy: EventLoopProxy<VxEvent>) -> Self {
		let window = Arc::new(window);
		let size = window.inner_size();

		let surface = gpu.instance.create_surface(window.clone())
			.expect("VxWindowStats> Critical: failed to create_surface.");

		let caps = surface.get_capabilities(&gpu.adapter);
		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: wgpu::TextureFormat::Rgba8Unorm,
			width: size.width.max(1),
			height: size.height.max(1),
			present_mode: wgpu::PresentMode::Fifo,
			alpha_mode: caps.alpha_modes[0],
			view_formats: vec![],
			desired_maximum_frame_latency: 1,
		};
		gpu.update_surface_config(&surface, &config);

		let renderer = VxRenderer::new(gpu, &config);

		let scene = VxScene::new();

		let input = VxInputState::new(
			VxMouseState::new(
				Default::default(),
				Default::default(),
				Default::default(),
			),
			VxKeyboardState::new("".into()),
			VxKeyModifierState::new(false, false, false),
		);

		Self {
			window,
			proxy,
			surface,
			config,
			renderer,
			scene,
			input,
			next_update_mode: VxDirtyCheckResult::All,
			is_dirty: true,
		}
	}

	pub(crate) fn resized_event(
		&mut self,
		gpu: &VxGpuResource,
		new_size: winit::dpi::PhysicalSize<u32>,
	) {
		if new_size.width > 0 && new_size.height > 0 {
			self.config.width = new_size.width;
			self.config.height = new_size.height;
			gpu.update_surface_config(&self.surface, &self.config);
			self.renderer.update_projection(
				gpu,
				VxMatrix4x4::orthographic(VxSize::from_u32(new_size.width, new_size.height)),
			);
		}
		self.is_dirty = true;
	}
	pub fn update_event(&mut self, res: &mut VxAppResource) {
		let input = &self.input;
		match self.next_update_mode {
			VxDirtyCheckResult::All => {
				let mut painter = VxPainter::new();
				self.scene.paint_event(res, &mut painter);
				self.scene.immediate_paint_event(res, input, &mut painter);
				// どうせ全部再描画だしImmediateバッファはクリアするから、同じバッファにまとめる
				self.take_and_set_vertices_to_renderer(res, &mut painter, VxRenderMode::Retained);
			}
			VxDirtyCheckResult::OnlyImmediate => {
				let mut painter = VxPainter::new();
				self.scene.immediate_paint_event(res, input, &mut painter);
				self.take_and_set_vertices_to_renderer(res, &mut painter, VxRenderMode::Immediate);
			}
			_ => { return; }
		}
		self.next_update_mode = VxDirtyCheckResult::None;
		
		res.textures.update_bind_group(&res.gpu);

		self.renderer.render(res, &self.surface);
	}

	fn take_and_set_vertices_to_renderer(&mut self, res: &mut VxAppResource, painter: &mut VxPainter, render_mode: VxRenderMode) {
		self.renderer.prepare_render(render_mode);

		let verts = std::mem::take(&mut painter.vertices);
		let sdf_verts = std::mem::take(&mut painter.sdf_verts);
		let tex_verts = std::mem::take(&mut painter.tex_verts);
		let text_data = std::mem::take(&mut painter.text_data);
		let text_verts = res.fonts.generate_text_vertices(&res.gpu, text_data);

		self.renderer.set_vertex_vertices(&res.gpu, render_mode, verts);
		self.renderer.set_sdf_vertices(&res.gpu, render_mode, sdf_verts);
		self.renderer.set_texture_vertices(&res.gpu, render_mode, tex_verts);
		self.renderer.set_text_vertices(&res.gpu, render_mode, text_verts);
	}

	pub(crate) fn check_dirty(&mut self) {
		let dirty = self.scene.check_dirty();
		if dirty != VxDirtyCheckResult::None || self.is_dirty {
			self.next_update_mode = if self.is_dirty { VxDirtyCheckResult::All } else { dirty };
			self.is_dirty = false;
			self.window.request_redraw();
		}
	}

	pub fn set_dirty(&mut self, dirty: bool) {
		self.is_dirty = dirty;
	}

	pub fn scale_factor(&self) -> f32 {
		self.window.scale_factor() as f32
	}

	pub fn finalize_init(&mut self) {
		// self.scene.refresh_spatial_index();
	}
}

pub(crate) trait VxWindowInternal: std::any::Any {
	fn stats(&self) -> &Option<VxWindowStats>;
	fn stats_mut(&mut self) -> &mut Option<VxWindowStats>;
	fn set_stats(&mut self, stat: VxWindowStats);
	fn window_attr(&self) -> &VxWindowAttributes;
	fn create_window_attr(&self) -> WindowAttributes {
		self.window_attr().create_window_attr()
	}
}

pub trait VxWindow: VxWindowInternal {
	/// ## VxWindow> events> init_event()
	/// Called during window initalization.
	/// #### Note: [`VxWindowInternal::stats`] will always return `Some(stats)` when this event is triggered.
	fn init_event(&mut self) {}
	fn has_immediate(&self) -> bool { false }

	fn update_event(&mut self, res: &mut VxAppResource) {
		if let Some(stat) = self.stats_mut() {
			stat.update_event(res);
		}
	}
	// Events
	fn chain_resize_event(&mut self, gpu: &VxGpuResource, new_size: winit::dpi::PhysicalSize<u32>) {
		if let Some(stat) = self.stats_mut() {
			stat.resized_event(gpu, new_size);
		}
	}

	fn handle_event(&mut self, event: &VxEvent) {
		match event {
			VxEvent::MousePressEvent { event } => {
				self.mouse_press_event(&event);
			}
			VxEvent::MouseReleaseEvent { event } => {
				self.mouse_release_event(&event);
			}
			VxEvent::MouseMoveEvent { event } => {
				self.mouse_move_event(&event);
			}
			VxEvent::MouseWheelEvent { event } => {
				self.mouse_wheel_event(&event);
			}
			VxEvent::KeyPressedEvent { event } => {
				self.key_press_event(&event);
			}
			VxEvent::KeyReleasedEvent { event } => {
				self.key_release_event(&event);
			}
			VxEvent::ResizeEvent { event } => {
				self.resize_event(&event);
			}
			VxEvent::ShowEvent { .. } => {
				self.show_event();
			}
			VxEvent::CloseEvent { .. } => {
				self.close_event();
			}
		}
	}

	fn mouse_press_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		if let Some(stat) = self.stats_mut() {
			return stat.scene.mouse_press_event(&event);
		}
		VxEventResult::Accept
	}
	fn mouse_release_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		if let Some(stat) = self.stats_mut() {
			return stat.scene.mouse_release_event(&event);
		}
		VxEventResult::Accept
	}
	fn mouse_move_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		if let Some(stat) = self.stats_mut() {
			return stat.scene.mouse_move_event(&event);
		}
		VxEventResult::Accept
	}
	fn mouse_wheel_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		if let Some(stat) = self.stats_mut() {
			return stat.scene.mouse_wheel_event(&event);
		}
		VxEventResult::Accept
	}
	fn key_press_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		if let Some(stat) = self.stats_mut() {
			return stat.scene.key_press_event(&event);
		}
		VxEventResult::Accept
	}
	fn key_release_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		if let Some(stat) = self.stats_mut() {
			return stat.scene.key_release_event(&event);
		}
		VxEventResult::Accept
	}

	fn resize_event(&mut self, event: &VxWindowEvent) -> VxEventResult {
		let _ = event;
		VxEventResult::Accept
	}
	fn show_event(&self) -> VxEventResult {
		VxEventResult::Accept
	}
	fn close_event(&self) -> VxEventResult {
		VxEventResult::Accept
	}
}

pub trait VxWindowExt: VxWindow {
	fn add_widget<W: VxWidget>(&mut self, widget: W) -> Option<VxWidgetHandler<W>> {
		if let Some(stats) = self.stats_mut() {
			return Some(stats.scene.add_widget(widget));
		} else {
			return None;
		}
	}
	// fn get_widget<W: VxWidget>(&self, handler: VxWidgetHandler<W>) -> Option<&W> {
	// 	if let Some(stats) = self.stats() {
	// 		stats.scene.get_widget(handler)
	// 	} else {
	// 		None
	// 	}
	// }
	// fn get_widget_mut<W: VxWidget>(&mut self, handler: VxWidgetHandler<W>) -> Option<&mut W> {
	// 	if let Some(stats) = self.stats_mut() {
	// 		stats.scene.get_widget_mut(handler)
	// 	} else {
	// 		None
	// 	}
	// }
	fn set_fixed_size(&self, size: Option<VxSize>) {
		if let Some(s) = self.stats() {
			VxWindowFunctions::set_fixed_size(&s.window, size);
		}
	}
	fn set_minimum_size(&self, size: Option<VxSize>) {
		if let Some(s) = self.stats() {
			VxWindowFunctions::set_minimum_size(&s.window, size);
		}
	}
	fn set_maximum_size(&self, size: Option<VxSize>) {
		if let Some(s) = self.stats() {
			VxWindowFunctions::set_maximum_size(&s.window, size);
		}
	}
	fn set_window_resizable(&self, resizable: bool) {
		if let Some(s) = self.stats() {
			VxWindowFunctions::set_window_resizable(&s.window, resizable);
		}
	}
	fn set_transparent(&self, transparent: bool) {
		if let Some(s) = self.stats() {
			VxWindowFunctions::set_transparent(&s.window, transparent);
		}
	}
	fn show_fullscreen(&self) {
		if let Some(s) = self.stats() {
			VxWindowFunctions::show_fullscreen(&s.window);
		}
	}
	fn show_normal(&self) {
		if let Some(s) = self.stats() {
			VxWindowFunctions::show_normal(&s.window);
		}
	}
	fn is_fullscreen(&self) -> bool {
		if let Some(s) = self.stats() {
			return VxWindowFunctions::is_fullscreen(&s.window);
		}
		false
	}
	fn close(&self) {
		let res = self.close_event();
		if res == VxEventResult::Ignore {
			return;
		}
		if let Some(s) = self.stats() {
			VxWindowFunctions::close(&s.window, &s.proxy);
		}
	}
	fn show(&self, window: Box<dyn VxWindowBuilder>) {
		if let Some(stat) = self.stats() {
			VxWindowFunctions::show(window, &stat.proxy);
		}
	}
	fn update(&mut self) {
		if let Some(stat) = self.stats_mut() {
			stat.set_dirty(true);
		}
	}
	fn set_window_layer(&self, layer: VxWindowLayer) {
		if let Some(s) = self.stats() {
			VxWindowFunctions::set_window_layer(&s.window, layer);
		}
	}
	fn set_window_minimizable(&self, minimizable: bool) {
		if let Some(s) = self.stats() {
			VxWindowFunctions::set_window_minimizable(&s.window, minimizable);
		}
	}
}
impl<T: VxWindow + ?Sized> VxWindowExt for T {}