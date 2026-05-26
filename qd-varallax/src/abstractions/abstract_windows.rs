use std::sync::Arc;

use winit::{
	dpi::LogicalSize,
	event_loop::EventLoopProxy,
	window::{
		Window,
		WindowAttributes,
		WindowLevel
	}
};

use crate::{
	abstractions::{
		abstract_widgets::{
			VxWidget,
			VxWidgetId
		},
		window_function::VxWindowFunctions
	},
	core::{
		gpu_resource::VxGpuResource,
		renderer::VxRenderer,
		resource::VxAppResources,
		scene::VxScene,
		systems::VxFontSystem,
		vx_event::{
			VxEvent,
			VxEventResult,
			VxKeyEvent,
			VxMouseEvent,
			VxWindowEvent
		}
	},
	painter::painter::VxPainter,
	types::{
		geometry::VxSize,
		transform::VxMatrix4x4
	}
};

#[derive(Clone, Debug, PartialEq)]
pub struct VxWindowAttributes {
	title: String,
	size: VxSize,
}

impl Default for VxWindowAttributes {
	fn default() -> Self {
		Self { title: "VxWindow".into(), size: VxSize::from_i32(1280, 720) }
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
			.with_inner_size(LogicalSize::new(
				self.size.width(),
				self.size.height(),
			)
		);
		attr
	}
}

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

	is_dirty: bool,
}

impl VxWindowStats {
	pub fn new(gpu: &VxGpuResource, window: Window, proxy: EventLoopProxy<VxEvent>) -> Self {
		let window = Arc::new(window);
		let size = window.inner_size();

		let surface = gpu.instance.create_surface(window.clone()).unwrap();

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

		Self {
			window,
			proxy,
			surface,
			config,
			renderer,
			scene,
			is_dirty: true,
		}
	}

	pub(crate) fn resized_event(&mut self, gpu: &VxGpuResource, new_size: winit::dpi::PhysicalSize<u32>) {
		if new_size.width > 0 && new_size.height > 0 {
			self.config.width = new_size.width;
			self.config.height = new_size.height;
			gpu.update_surface_config(&self.surface, &self.config);
			self.renderer.update_projection(
				gpu,
				VxMatrix4x4::orthographic(
					VxSize::from_u32(
						new_size.width,
						new_size.height
					)
				)
			);
		}
	}
	pub fn update_event(&mut self, res: &mut VxAppResources) {
		let mut painter = VxPainter::new();
		self.scene.paint_event(res, &mut painter);
		res.textures.update_bind_group(&res.gpu);

		let verts = std::mem::take(&mut painter.vertices);
		let sdf_verts = std::mem::take(&mut painter.sdf_verts);
		let tex_verts = std::mem::take(&mut painter.tex_verts);
		let text_data = std::mem::take(&mut painter.text_data);
		let text_verts = VxFontSystem::embed_atlas_id(res, text_data);

		// 頂点のみデータを描画
		self.renderer.set_vertex_vertices(&res.gpu, verts);

		self.renderer.set_sdf_vertices(&res.gpu, sdf_verts);

		// テクスチャデータを描画
		self.renderer.set_texture_vertices(&res.gpu, tex_verts);

		self.renderer.set_text_vertices(&res.gpu, text_verts);

		self.renderer.render(res, &self.surface);
	}
	pub(crate) fn check_dirty(&mut self) {
		if self.scene.check_dirty() || self.is_dirty {
			self.set_dirty(false);
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
		self.scene.refresh_spatial_index();
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

	fn update_event(&mut self, res: &mut VxAppResources) {
		if let Some(stat) = self.stats_mut() {
			stat.update_event(res);
		}
	}

	//wrappers
	fn send_widget_to_scene(&mut self, widget: Box<dyn VxWidget>) -> Option<VxWidgetId> {
		if let Some(stats) = self.stats_mut(){
			Some(stats.scene.add_widget(widget))
		} else {
			None
		}
	}
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
		if res == VxEventResult::Ignore { return }
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
			return stat.scene.mouse_press_event(&event)
		}
		VxEventResult::Accept
	}
	fn mouse_release_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		if let Some(stat) = self.stats_mut() {
			return stat.scene.mouse_release_event(&event)
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
			return stat.scene.mouse_wheel_event(&event)
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

//add_widgetのための拡張
pub trait VxWindowExt: VxWindow {
	fn add_widget<W: VxWidget + 'static>(&mut self, widget: W) -> Option<VxWidgetId> {
		self.send_widget_to_scene(Box::new(widget))
	}
}

impl<T: VxWindow + ?Sized> VxWindowExt for T {}