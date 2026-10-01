use std::sync::Arc;

use winit::{
	dpi::LogicalSize,
	event_loop::EventLoopProxy,
	window::{Window, WindowAttributes, WindowLevel},
};

use crate::{
	abstracts::{
		abstract_widgets::{
			VxWidget, VxWidgetHandler, VxWidgetId
		}, window_function::VxWindowFunctions
	}, core::{
		gpu::VxGpuResource,
		renderer::VxRenderer,
		resource::VxAppResource,
		scene::VxScene,
	}, painter::painter::VxPainter, types::{
		event::{
			VxEvent,
			VxEventResult,
			VxKeyEvent,
			VxMouseEvent,
			VxWindowEvent
		}, geometry::VxSize, input::{
			VxInputState,
			VxKeyModifierState,
			VxKeyboardState,
			VxMouseState
		}, render_commands::{
			VxDirtyCheckResult,
			VxRenderMode
		}, style::{VxColorPalette, VxThemeMode}, transform::VxMatrix4x4
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

pub struct VxWindowContext {
	pub(crate) window: Arc<Window>,
	pub(crate) proxy: EventLoopProxy<VxEvent>,
	surface: wgpu::Surface<'static>,
	config: wgpu::SurfaceConfiguration,
	renderer: VxRenderer,
	scene: VxScene,
	painter: VxPainter,
	palette: VxColorPalette,
	input: VxInputState,

	next_update_mode: VxDirtyCheckResult,
	is_dirty: bool,
	window_size: VxSize,
}

impl VxWindowContext {
	pub fn new(gpu: &VxGpuResource, window: Arc<Window>, proxy: EventLoopProxy<VxEvent>) -> Self {
		window.set_ime_allowed(true);
		let size = window.inner_size();

		let surface = gpu.instance.create_surface(window.clone())
			.expect("VxWindowContext> Critical: failed to create_surface.");

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

		let theme = window.theme()
			.map(|theme| {
				match theme {
					winit::window::Theme::Dark => VxThemeMode::Dark,
					winit::window::Theme::Light => VxThemeMode::Light,
				}
			})
			.unwrap_or_else(|| VxThemeMode::Dark);

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
			palette: VxColorPalette::from_theme(theme),
			input,
			next_update_mode: VxDirtyCheckResult::All,
			is_dirty: true,
			window_size: VxSize::default(),
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
			self.window_size = new_size;
			self.scene.resized_event(new_size);
		}
		self.is_dirty = true;
		VxEventResult::Ignore
	}
	pub fn update_event(&mut self, res: &mut VxAppResource) {
		match self.next_update_mode {
			VxDirtyCheckResult::All => {
				self.scene.paint_event(res, &mut self.painter, &self.palette);
				self.take_and_set_vertices_to_renderer(res, VxRenderMode::Retained);
				self.scene.immediate_paint_event(res, &self.input, &mut self.painter);
				// バッファをまとめたかったが、Immediate描画のバッファがRetainedに残存してしまうので仕方なく分ける
				self.take_and_set_vertices_to_renderer(res, VxRenderMode::Immediate);
			}
			VxDirtyCheckResult::OnlyImmediate => {
				self.scene.immediate_paint_event(res, &self.input, &mut self.painter);
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

	pub(crate) fn check_dirty(&mut self, res: &mut VxAppResource) -> bool {
		let dirty = self.scene.check_dirty(res, self.window_size);
		if dirty != VxDirtyCheckResult::None || self.is_dirty {
			self.next_update_mode = if self.is_dirty { VxDirtyCheckResult::All } else { dirty };
			self.is_dirty = false;
			self.window.request_redraw();
			true
		} else {
			false
		}
	}

	#[inline]
	pub fn set_dirty(&mut self, dirty: bool) {
		self.is_dirty = dirty;
	}
	#[inline]
	pub fn scale_factor(&self) -> f32 {
		self.window.scale_factor() as f32
	}
	#[inline]
	pub fn scene(&self) -> &VxScene {
		&self.scene
	}
	#[inline]
	pub fn scene_mut(&mut self) -> &mut VxScene {
		&mut self.scene
	}
	#[inline]
	pub fn theme(&self) -> VxThemeMode {
		self.window.theme()
			.map(|theme| {
				match theme {
					winit::window::Theme::Dark => VxThemeMode::Dark,
					winit::window::Theme::Light => VxThemeMode::Light,
				}
			})
			.unwrap_or_else(|| VxThemeMode::Dark)
	}
	#[inline]
	pub fn sync_system_theme(&mut self) {
		self.palette = VxColorPalette::from_theme(self.theme())
	}
	#[inline]
	pub fn set_theme(&mut self, theme: VxThemeMode) {
		if self.palette.theme() == theme { return; }
		self.palette = VxColorPalette::from_theme(theme);
		self.is_dirty = true;
		self.next_update_mode = VxDirtyCheckResult::All;
	}
}

pub trait VxWindowAccessor: std::any::Any {
	fn context(&self) -> &Option<VxWindowContext>;
	fn context_mut(&mut self) -> &mut Option<VxWindowContext>;
	fn set_context(&mut self, context: VxWindowContext);
	fn window_attr(&self) -> &VxWindowAttributes;
	fn create_window_attr(&self) -> WindowAttributes {
		self.window_attr().create_window_attr()
	}
}

pub trait VxWindow: VxWindowAccessor {
	/// ## VxWindow> events> init_event()
	/// Called during window initalization.
	/// #### Note: [`VxWindowAccessor::context`] will always return `Some(context)` when this event is triggered.
	fn init_event(&mut self) {}
	fn has_immediate(&self) -> bool {
		self.context()
			.as_ref()
			.map(|ctx| ctx.scene().has_immediate_widget())
			.unwrap_or_else(|| false)
	}

	fn update_event(&mut self, res: &mut VxAppResource) {
		if let Some(ctx) = self.context_mut() {
			ctx.update_event(res);
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
		self.context_mut().as_mut()
			.map_or(VxEventResult::Accept,	|ctx| ctx.scene.mouse_press_event(event))
	}
	fn mouse_release_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		self.context_mut().as_mut()
			.map_or(VxEventResult::Accept,	|ctx| ctx.scene.mouse_release_event(event))
	}
	fn mouse_move_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		self.context_mut().as_mut()
			.map_or(VxEventResult::Accept,	|ctx| ctx.scene.mouse_move_event(event))
	}
	fn mouse_wheel_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		self.context_mut().as_mut()
			.map_or(VxEventResult::Accept,	|ctx| ctx.scene.mouse_wheel_event(event))
	}
	fn key_press_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		self.context_mut().as_mut()
			.map_or(VxEventResult::Accept,	|ctx| ctx.scene.key_press_event(event))
	}
	fn key_release_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		self.context_mut().as_mut()
			.map_or(VxEventResult::Accept,	|ctx| ctx.scene.key_release_event(event))
	}

	fn resize_event(&mut self, gpu: &VxGpuResource, event: &VxWindowEvent) -> VxEventResult {
		self.context_mut().as_mut()
			.map_or(VxEventResult::Accept,	|ctx| ctx.resized_event(gpu, event.size()))
	}
	fn show_event(&self) -> VxEventResult {
		VxEventResult::Accept
	}
	fn close_event(&self) -> VxEventResult {
		VxEventResult::Accept
	}

	fn theme_changed_event(&mut self, new_theme: VxThemeMode) {
		self.context_mut().as_mut()
			.map(|ctx| ctx.set_theme(new_theme));
	}
}

pub trait VxWindowExt: VxWindow {
	#[inline]
	fn add_widget<W: VxWidget>(&mut self, widget: W) -> Option<VxWidgetHandler<W>> {
		self.context_mut().as_mut()
			.map(|ctx| ctx.scene.add_widget(widget))
	}
	#[inline]
	fn add_widgets(&mut self, widgets: impl IntoIterator<Item = Box<dyn VxWidget>>) -> Option<Vec<VxWidgetId>> {
		self.context_mut().as_mut()
			.and_then(|ctx| {
				Some(widgets.into_iter()
					.map(|w| ctx.scene.add_widget_box(w))
					.collect::<Vec<_>>())
			})
	}
	#[inline]
	fn get_widget<W: VxWidget>(&self, handler: VxWidgetHandler<W>) -> Option<&W> {
		self.context().as_ref()
			.and_then(|ctx| ctx.scene.get_widget(handler))
	}
	#[inline]
	fn get_widget_mut<W: VxWidget>(&mut self, handler: VxWidgetHandler<W>) -> Option<&mut W> {
		self.context_mut().as_mut()
			.and_then(|ctx| ctx.scene.get_widget_mut(handler))
	}
	#[inline]
	fn remove_widget<W: VxWidget>(&mut self, handler: VxWidgetHandler<W>) -> Option<W> {
		self.context_mut().as_mut()?.scene_mut().remove_widget(handler)
	}
	#[inline]
	fn remove_widget_id(&mut self, id: VxWidgetId) -> Option<Box<dyn VxWidget>> {
		self.context_mut().as_mut()?.scene_mut().remove_widget_id(id)
	}
	#[inline]
	fn set_fixed_size(&self, size: Option<VxSize>) {
		self.context().as_ref()
			.map(|ctx| VxWindowFunctions::set_fixed_size(&ctx.window, size));
	}
	#[inline]
	fn set_minimum_size(&self, size: Option<VxSize>) {
		self.context().as_ref()
			.map(|ctx| VxWindowFunctions::set_minimum_size(&ctx.window, size));
	}
	#[inline]
	fn set_maximum_size(&self, size: Option<VxSize>) {
		self.context().as_ref()
			.map(|ctx| VxWindowFunctions::set_maximum_size(&ctx.window, size));
	}
	#[inline]
	fn set_window_resizable(&self, resizable: bool) {
		self.context().as_ref()
			.map(|ctx| VxWindowFunctions::set_window_resizable(&ctx.window, resizable));
	}
	#[inline]
	fn set_transparent(&self, transparent: bool) {
		self.context().as_ref()
			.map(|ctx| VxWindowFunctions::set_transparent(&ctx.window, transparent));
	}
	#[inline]
	fn show_fullscreen(&self) {
		self.context().as_ref()
			.map(|ctx| VxWindowFunctions::show_fullscreen(&ctx.window));
	}
	#[inline]
	fn show_normal(&self) {
		self.context().as_ref()
			.map(|ctx| VxWindowFunctions::show_normal(&ctx.window));
	}
	#[inline]
	fn is_fullscreen(&self) -> bool {
		self.context().as_ref()
			.map_or(false, |ctx| VxWindowFunctions::is_fullscreen(&ctx.window))
	}
	#[inline]
	fn close(&self) {
		let res = self.close_event();
		if res == VxEventResult::Ignore {
			return;
		}
		self.context().as_ref()
			.map(|ctx| VxWindowFunctions::close(&ctx.window, &ctx.proxy));
	}
	#[inline]
	fn show(&self, window: Box<dyn VxWindowBuilder>) {
		self.context().as_ref()
			.map(|ctx| VxWindowFunctions::show(window, &ctx.proxy));
	}
	#[inline]
	fn update(&mut self) {
		self.context_mut().as_mut()
			.map(|ctx| ctx.set_dirty(true));
	}
	#[inline]
	fn set_window_layer(&self, layer: VxWindowLayer) {
		self.context().as_ref()
			.map(|ctx| VxWindowFunctions::set_window_layer(&ctx.window, layer));
	}
	#[inline]
	fn set_window_minimizable(&self, minimizable: bool) {
		self.context().as_ref()
			.map(|ctx| VxWindowFunctions::set_window_minimizable(&ctx.window, minimizable));
	}
	#[inline]
	fn theme(&self) -> VxThemeMode {
		self.context().as_ref()
			.map(|ctx| ctx.theme()).unwrap_or_else(|| VxThemeMode::Dark)
	}
}
impl<T: VxWindow + ?Sized> VxWindowExt for T {}