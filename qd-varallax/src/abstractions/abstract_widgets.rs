use crate::{
	abstractions::abstract_layouts::VxAlignment, core::{gpu_resource::VxGpuResource, systems::VxTextureSystem, vx_event::{
		VxEventResult,
		VxKeyEvent,
		VxMouseEvent
	}}, painter::painter::VxPainter, types::{
		genelational_vector::VxGenIndex, geometry::{
			VxRect,
			VxSize,
			VxVec2
		},
		transform::{
			VxAngle,
			VxTransform
		}
	}, vx_signal
};



#[derive(Hash, Copy, Clone, PartialEq, Eq, Debug)]
pub struct VxWidgetId {
	pub id: VxGenIndex,
}

impl VxWidgetId {
	#[inline]
	pub fn new(id: VxGenIndex) -> Self {
		Self { id }
	}
	#[inline]
	pub fn id(&self) -> VxGenIndex {
		self.id
	}
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub struct VxWidgetHandler<W: VxWidget> {
	id: VxWidgetId,
	_marker: std::marker::PhantomData<W>
}

impl<W: VxWidget> VxWidgetHandler<W> {
	pub fn new(id: VxWidgetId) -> Self {
		Self {
			id,
			_marker: std::marker::PhantomData,
		}
	}
	pub fn id(&self) -> VxWidgetId { self.id }
}

pub struct VxWidgetStats {
	id: Option<VxWidgetId>,
	transform: VxTransform,
	visible: bool,
	z_value: i32,
	dirty: bool,
	parent: Option<VxWidgetId>,
	children: Vec<VxWidgetId>,
	alignment: VxAlignment,
	block_signal: bool,

	children_widgets: Vec<Box<dyn VxWidget>>,
}

impl VxWidgetStats {
	pub fn new(parent: Option<VxWidgetId>) -> Self {
		Self {
			id: None,
			transform: VxTransform::default(),
			visible: true,
			z_value: 0,
			dirty: false,
			parent,
			children: Vec::new(),
			alignment: VxAlignment::default(),
			block_signal: false,
			children_widgets: vec![],
		}
	}
	// getters
	#[inline]
	pub fn widget_id(&self) -> &Option<VxWidgetId> { &self.id }
	#[inline]
	pub fn parent(&self) -> &Option<VxWidgetId> { &self.parent }
	#[inline]
	pub fn children(&self) -> &Vec<VxWidgetId> { &self.children }
	#[inline]
	pub fn pos(&self) -> VxVec2 { self.transform().pos() }
	#[inline]
	pub fn angle(&self) -> VxAngle { self.transform().rotation() }
	#[inline]
	pub fn scale(&self) -> VxSize { self.transform().scale() }
	#[inline]
	pub fn center_pivot(&self) -> VxVec2 { self.transform().pivot() }
	#[inline]
	pub fn transform(&self) -> VxTransform { self.transform }
	#[inline]
	pub fn is_visible(&self) -> bool { self.visible }
	#[inline]
	pub fn z_value(&self) -> i32 { self.z_value }
	#[inline]
	pub fn is_dirty(&self) -> bool { self.dirty }
	#[inline]
	pub fn alignment(&self) -> VxAlignment { self.alignment }
	#[inline]
	pub fn is_block_signal(&self) -> bool { self.block_signal }
	#[inline]
	pub(crate) fn children_widgets(&mut self) -> Vec<Box<dyn VxWidget>> {
		std::mem::take(&mut self.children_widgets)
	}

	// setters
	#[inline]
	pub(crate) fn set_widget_id(&mut self, id: VxWidgetId) { self.id = Some(id); }
	#[inline]
	pub fn set_pos(&mut self, pos: VxVec2) {
	self.transform.set_pos(pos);
		self.set_dirty(true);
	}
	#[inline]
	pub fn set_angle(&mut self, angle: VxAngle) {
		self.transform.set_rotation(angle);
		self.set_dirty(true);
	}
	#[inline]
	pub fn set_scale(&mut self, scale: VxSize) {
		self.transform.set_scale(scale);
		self.set_dirty(true);
	}
	#[inline]
	pub fn set_center_pivot(&mut self, pivot: VxVec2) {
		self.transform.set_center_pivot(pivot);
		self.set_dirty(true);
	}
	#[inline]
	pub fn set_transform(&mut self, transform: VxTransform) {
		self.transform = transform;
		self.set_dirty(true);
	}
	#[inline]
	pub fn set_parent(&mut self, parent: VxWidgetId) { self.parent = Some(parent); }
	#[inline]
	pub fn set_visible(&mut self, visible: bool) {
		self.visible = visible;
		self.set_dirty(true);
	}
	#[inline]
	pub fn set_z_value(&mut self, z: i32) {
		if self.z_value == z { return; }
		self.z_value = z;
		self.set_dirty(true);
	}
	#[inline]
	pub fn set_dirty(&mut self, dirty: bool) { self.dirty = dirty; }
	#[inline]
	pub fn set_alignment(&mut self, alignment: VxAlignment) {
		self.alignment = alignment;
		self.set_dirty(true);
	}
	#[inline]
	pub fn add_child(&mut self, child: VxWidgetId) { self.children.push(child); }
	#[inline]
	pub fn add_child_widget<W: VxWidget>(&mut self, child: W) {
		self.children_widgets.push(Box::new(child));
	}
}

pub(crate) trait VxWidgetInternal: std::any::Any {
	fn stats(&self) -> &VxWidgetStats;
	fn stats_mut(&mut self) -> &mut VxWidgetStats;
	fn as_any(&self) -> &dyn std::any::Any;
	fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub trait VxWidget: VxWidgetInternal {
	fn bounding_rect(&self) -> VxRect;
	fn paint(&mut self, painter: &mut VxPainter);

	// Stats Wrapping
	#[inline]
	fn widget_id(&self) -> &Option<VxWidgetId> { &self.stats().widget_id() }
	#[inline]
	fn parent(&self) -> &Option<VxWidgetId> { &self.stats().parent() }
	#[inline]
	fn children(&self) -> &Vec<VxWidgetId> { &self.stats().children() }
	#[inline]
	fn pos(&self) -> VxVec2 { self.stats().pos() }
	#[inline]
	fn angle(&self) -> VxAngle { self.stats().angle() }
	#[inline]
	fn scale(&self) -> VxSize { self.stats().scale() }
	#[inline]
	fn transform(&self) -> VxTransform { self.stats().transform() }
	#[inline]
	fn is_visible(&self) -> bool { self.stats().is_visible() }
	#[inline]
	fn z_value(&self) -> i32 { self.stats().z_value() }
	#[inline]
	fn is_dirty(&self) -> bool { self.stats().is_dirty() }
	#[inline]
	fn is_block_signal(&self) -> bool { self.stats().is_block_signal() }

	#[inline]
	fn set_parent(&mut self, parent: VxWidgetId) { self.stats_mut().set_parent(parent); }
	#[inline]
	fn set_pos(&mut self, pos: VxVec2) { self.stats_mut().set_pos(pos); }
	#[inline]
	fn set_angle(&mut self, angle: VxAngle) { self.stats_mut().set_angle(angle); }
	#[inline]
	fn set_scale(&mut self, scale: VxSize) { self.stats_mut().set_scale(scale); }
	#[inline]
	fn set_center_pivot(&mut self, pivot: VxVec2) { self.stats_mut().set_center_pivot(pivot); }
	#[inline]
	fn set_transform(&mut self, transform: VxTransform) { self.stats_mut().set_transform(transform); }
	#[inline]
	fn set_visible(&mut self, visible: bool) { self.stats_mut().set_visible(visible); }
	#[inline]
	fn set_z_value(&mut self, z: i32) { self.stats_mut().set_z_value(z); }
	#[inline]
	fn set_dirty(&mut self, dirty: bool) { self.stats_mut().set_dirty(dirty); }

	// Events
	/// マウスが押されたときのイベント
	fn mouse_press_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let _ = event;
		VxEventResult::Ignore
	}
	/// マウスが離れたときのイベント
	fn mouse_release_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let _ = event;
		VxEventResult::Ignore
	}
	/// マウスホバー開始時イベント
	fn mouse_enter_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let _ = event;
		VxEventResult::Ignore
	}
	/// マウスホバー終了時イベント
	fn mouse_leave_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let _ = event;
		VxEventResult::Ignore
	}
	/// マウスがウィジェット上で動いたイベント
	fn mouse_move_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let _ = event;
		VxEventResult::Ignore
	}
	/// マウスホイールがウィジェット上で回転したイベント
	fn mouse_wheel_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let _ = event;
		VxEventResult::Ignore
	}
	/// キーが押されたときのイベント
	fn key_press_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		let _ = event;
		VxEventResult::Ignore
	}
	/// キーが離れたときのイベント
	fn key_release_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		let _ = event;
		VxEventResult::Ignore
	}
	fn register_texture_event(&mut self, gpu: &VxGpuResource, system: &mut VxTextureSystem) {
		let _ = gpu;
		let _ = system;
	}
}

// signals
vx_signal!(pub struct VxHoveredSignal >> VxVec2);
vx_signal!(pub struct VxLeavedSignal >> VxVec2);
vx_signal!(pub struct VxMovedSignal >> VxVec2);
vx_signal!(pub struct VxPressedSignal >> VxVec2);
vx_signal!(pub struct VxReleasedSignal >> VxVec2);

#[macro_export]
macro_rules! vx_widget_signals {
	($vis:vis struct $name:ident { $($field:ident : $sig_name:ident >> $msg:ty),* $(,)? }) => {
		$(
			$crate::vx_signal!($vis struct $sig_name >> $msg);
		)*

		$vis struct $name<Sender: ?Sized> {
			pub pressed: crate::abstractions::abstract_widgets::VxPressedSignal<Sender>,
			pub released: crate::abstractions::abstract_widgets::VxReleasedSignal<Sender>,
			pub hovered: crate::abstractions::abstract_widgets::VxHoveredSignal<Sender>,
			pub leaved: crate::abstractions::abstract_widgets::VxLeavedSignal<Sender>,
			pub moved: crate::abstractions::abstract_widgets::VxMovedSignal<Sender>,
			$( pub $field : $sig_name<Sender>, )*
			_marker: std::marker::PhantomData<Sender>,
		}

		impl<Sender: ?Sized> $name<Sender> {
			pub fn new() -> Self {
				Self {
					pressed: crate::abstractions::abstract_widgets::VxPressedSignal::new(),
					released: crate::abstractions::abstract_widgets::VxReleasedSignal::new(),
					hovered: crate::abstractions::abstract_widgets::VxHoveredSignal::new(),
					leaved: crate::abstractions::abstract_widgets::VxLeavedSignal::new(),
					moved: crate::abstractions::abstract_widgets::VxMovedSignal::new(),
					$( $field : $sig_name::new(), )*
					_marker: std::marker::PhantomData,
				}
			}
		}
	};
}

vx_widget_signals!(pub struct VxDefaultSignals {});