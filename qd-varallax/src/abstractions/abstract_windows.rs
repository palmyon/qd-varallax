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
	painter: VxPainter,
	input: VxInputState,

	next_update_mode: VxDirtyCheckResult,
	is_dirty: bool,
}

impl VxWindowStats {
	pub fn new(gpu: &VxGpuResource, window: Arc<Window>, proxy: EventLoopProxy<VxEvent>) -> Self {
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
			scene: VxScene::new(),
			painter: VxPainter::new(),
			input,
			next_update_mode: VxDirtyCheckResult::All,
			is_dirty: true,
		}
	}

	pub(crate) fn resized_event(
		&mut self,
		gpu: &VxGpuResource,
		new_size: VxSize,
	) -> VxEventResult {
		if !new_size.is_empty() {
			self.config.width = new_size.width_u32();
			self.config.height = new_size.height_u32();
			gpu.update_surface_config(&self.surface, &self.config);
			self.renderer.update_projection(
				gpu,
				VxMatrix4x4::orthographic(new_size),
			);
		}
		self.is_dirty = true;
		VxEventResult::Ignore
	}
	pub fn update_event(&mut self, res: &mut VxAppResource) {
		let input = &self.input;
		match self.next_update_mode {
			VxDirtyCheckResult::All => {
				self.scene.paint_event(res, &mut self.painter);
				self.scene.immediate_paint_event(res, input, &mut self.painter);
				// どうせ全部再描画だしImmediateバッファはクリアするから、同じバッファにまとめる
				self.take_and_set_vertices_to_renderer(res, VxRenderMode::Retained);
			}
			VxDirtyCheckResult::OnlyImmediate => {
				self.scene.immediate_paint_event(res, input, &mut self.painter);
				self.take_and_set_vertices_to_renderer(res, VxRenderMode::Immediate);
			}
			_ => { return; }
		}
		self.next_update_mode = VxDirtyCheckResult::None;
		
		res.textures.update_bind_group(&res.gpu);

		self.renderer.render(res, &self.surface);
	}

	fn take_and_set_vertices_to_renderer(&mut self, res: &mut VxAppResource, render_mode: VxRenderMode) {
		self.renderer.prepare_render(render_mode);

		let verts = std::mem::take(&mut self.painter.vertices);
		let sdf_verts = std::mem::take(&mut self.painter.sdf_verts);
		let tex_verts = std::mem::take(&mut self.painter.tex_verts);
		let text_data = std::mem::take(&mut self.painter.text_data);
		let text_verts = res.fonts.generate_text_vertices(&res.gpu, text_data);

		self.renderer.set_vertices(&res.gpu, render_mode, verts);
		self.renderer.set_vertices(&res.gpu, render_mode, sdf_verts);
		self.renderer.set_vertices(&res.gpu, render_mode, tex_verts);
		self.renderer.set_vertices(&res.gpu, render_mode, text_verts);
	}

	pub(crate) fn check_dirty(&mut self) -> bool {
		let dirty = self.scene.check_dirty();
		if dirty != VxDirtyCheckResult::None || self.is_dirty {
			self.next_update_mode = if self.is_dirty { VxDirtyCheckResult::All } else { dirty };
			self.is_dirty = false;
			self.window.request_redraw();
			true
		} else {
			false
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

pub trait VxWindowInternal: std::any::Any {
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

	fn handle_event(&mut self, gpu: &VxGpuResource, event: &VxEvent) {
		match event {
			VxEvent::MousePressEvent { event } => {
				self.mouse_press_event(event);
			}
			VxEvent::MouseReleaseEvent { event } => {
				self.mouse_release_event(event);
			}
			VxEvent::MouseMoveEvent { event } => {
				self.mouse_move_event(event);
			}
			VxEvent::MouseWheelEvent { event } => {
				self.mouse_wheel_event(event);
			}
			VxEvent::KeyPressedEvent { event } => {
				self.key_press_event(event);
			}
			VxEvent::KeyReleasedEvent { event } => {
				self.key_release_event(event);
			}
			VxEvent::ResizeEvent { event } => {
				self.resize_event(gpu, event);
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
		self.stats_mut().as_mut()
			.map_or(VxEventResult::Accept,	|stats| stats.scene.mouse_press_event(event))
	}
	fn mouse_release_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		self.stats_mut().as_mut()
			.map_or(VxEventResult::Accept,	|stats| stats.scene.mouse_release_event(event))
	}
	fn mouse_move_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		self.stats_mut().as_mut()
			.map_or(VxEventResult::Accept,	|stats| stats.scene.mouse_move_event(event))
	}
	fn mouse_wheel_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		self.stats_mut().as_mut()
			.map_or(VxEventResult::Accept,	|stats| stats.scene.mouse_wheel_event(event))
	}
	fn key_press_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		self.stats_mut().as_mut()
			.map_or(VxEventResult::Accept,	|stats| stats.scene.key_press_event(event))
	}
	fn key_release_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		self.stats_mut().as_mut()
			.map_or(VxEventResult::Accept,	|stats| stats.scene.key_release_event(event))
	}

	fn resize_event(&mut self, gpu: &VxGpuResource, event: &VxWindowEvent) -> VxEventResult {
		self.stats_mut().as_mut()
			.map_or(VxEventResult::Accept,	|stats| stats.resized_event(gpu, event.size()))
	}
	fn show_event(&self) -> VxEventResult {
		VxEventResult::Accept
	}
	fn close_event(&self) -> VxEventResult {
		VxEventResult::Accept
	}
}

pub trait VxWindowExt: VxWindow {
	#[inline]
	fn add_widget<W: VxWidget>(&mut self, widget: W) -> Option<VxWidgetHandler<W>> {
		self.stats_mut().as_mut()
			.map(|stats| stats.scene.add_widget(widget))
	}
	#[inline]
	fn get_widget<W: VxWidget>(&self, handler: VxWidgetHandler<W>) -> Option<&W> {
		self.stats().as_ref()
			.and_then(|stats| stats.scene.get_widget(handler))
	}
	#[inline]
	fn get_widget_mut<W: VxWidget>(&mut self, handler: VxWidgetHandler<W>) -> Option<&mut W> {
		self.stats_mut().as_mut()
			.and_then(|stats| stats.scene.get_widget_mut(handler))
	}
	#[inline]
	fn set_fixed_size(&self, size: Option<VxSize>) {
		self.stats().as_ref()
			.map(|stats| VxWindowFunctions::set_fixed_size(&stats.window, size));
	}
	#[inline]
	fn set_minimum_size(&self, size: Option<VxSize>) {
		self.stats().as_ref()
			.map(|stats| VxWindowFunctions::set_minimum_size(&stats.window, size));
	}
	#[inline]
	fn set_maximum_size(&self, size: Option<VxSize>) {
		self.stats().as_ref()
			.map(|stats| VxWindowFunctions::set_maximum_size(&stats.window, size));
	}
	#[inline]
	fn set_window_resizable(&self, resizable: bool) {
		self.stats().as_ref()
			.map(|stats| VxWindowFunctions::set_window_resizable(&stats.window, resizable));
	}
	#[inline]
	fn set_transparent(&self, transparent: bool) {
		self.stats().as_ref()
			.map(|stats| VxWindowFunctions::set_transparent(&stats.window, transparent));
	}
	#[inline]
	fn show_fullscreen(&self) {
		self.stats().as_ref()
			.map(|stats| VxWindowFunctions::show_fullscreen(&stats.window));
	}
	#[inline]
	fn show_normal(&self) {
		self.stats().as_ref()
			.map(|stats| VxWindowFunctions::show_normal(&stats.window));
	}
	#[inline]
	fn is_fullscreen(&self) -> bool {
		self.stats().as_ref()
			.map_or(false, |stats| VxWindowFunctions::is_fullscreen(&stats.window))
	}
	#[inline]
	fn close(&self) {
		let res = self.close_event();
		if res == VxEventResult::Ignore {
			return;
		}
		self.stats().as_ref()
			.map(|stats| VxWindowFunctions::close(&stats.window, &stats.proxy));
	}
	#[inline]
	fn show(&self, window: Box<dyn VxWindowBuilder>) {
		self.stats().as_ref()
			.map(|stats| VxWindowFunctions::show(window, &stats.proxy));
	}
	#[inline]
	fn update(&mut self) {
		self.stats_mut().as_mut()
			.map(|stats| stats.set_dirty(true));
	}
	#[inline]
	fn set_window_layer(&self, layer: VxWindowLayer) {
		self.stats().as_ref()
			.map(|stats| VxWindowFunctions::set_window_layer(&stats.window, layer));
	}
	#[inline]
	fn set_window_minimizable(&self, minimizable: bool) {
		self.stats().as_ref()
			.map(|stats| VxWindowFunctions::set_window_minimizable(&stats.window, minimizable));
	}
}
impl<T: VxWindow + ?Sized> VxWindowExt for T {}