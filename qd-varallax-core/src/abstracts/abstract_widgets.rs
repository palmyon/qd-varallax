use std::{ops::Deref, rc::Rc};

use crate::{
	abstracts::abstract_layouts::{VxBoundingRectCreator, VxBoxLayout, VxSizeHint, VxSpatialLayout}, core::{
		gpu::VxGpuResource,
		systems::VxTextureSystem,
	}, painter::painter::VxPainter, types::{
		event::{
			VxEventResult,
			VxKeyEvent,
			VxMouseEvent
		}, gen_vector::{
			VxGenIndex, VxGenIndexWrapper
		}, geometry::{
			VxRect,
			VxSize,
			VxVec2
		}, input::VxInputState, render_commands::VxRenderMode, style::VxColorPalette, transform::{
			VxAngle,
			VxTransform
		}
	}, vx_signal
};



#[derive(Hash, Copy, Clone, PartialEq, Eq, Debug)]
pub struct VxWidgetId {
	id: VxGenIndex,
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

impl VxGenIndexWrapper for VxWidgetId {
	#[inline]
	fn new_invalid() -> Self {
		Self { id: VxGenIndex::new_invalid() }
	}
	#[inline]
	fn is_valid(&self) -> bool {
		self.id.is_valid()
	}
	#[inline]
	fn raw_index(&self) -> usize {
		self.id.raw_index()
	}
	#[inline]
	fn raw_index_u32(&self) -> u32 {
		self.id.raw_index_u32()
	}
	#[inline]
	fn as_gen_index(&self) -> VxGenIndex {
		self.id.as_gen_index()
	}
}

#[derive(Hash, Clone, Copy, PartialEq, Eq, Debug)]
pub struct VxWidgetHandler<W: VxWidget> {
	id: VxWidgetId,
	_marker: std::marker::PhantomData<W>
}
impl<W: VxWidget> Deref for VxWidgetHandler<W> {
	type Target = VxWidgetId;
	fn deref(&self) -> &Self::Target {
		&self.id
	}
}

impl<W: VxWidget> VxWidgetHandler<W> {
	#[inline]
	pub fn new(id: VxWidgetId) -> Self {
		Self {
			id,
			_marker: std::marker::PhantomData,
		}
	}
	#[inline]
	pub fn id(&self) -> VxWidgetId { self.id }
}

bitflags::bitflags!{
	#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, Default)]
	pub struct VxDirtyFlag: u32 {
		const CLEAN         = 0;
		const REPAINT       = 1 << 0;
		const LAYOUT        = 1 << 1;
		const REBUILD_ALL   = Self::CLEAN.bits() | Self::REPAINT.bits() | Self::LAYOUT.bits();
	}
}

#[derive(Clone)]
pub struct VxDirtyCommandSender {
	handler: Rc<dyn Fn(VxWidgetId, VxDirtyFlag)>,
}
impl VxDirtyCommandSender {
	#[inline]
	pub fn new(handler: impl Fn(VxWidgetId, VxDirtyFlag) + 'static) -> Self {
		Self { handler: Rc::new(handler) }
	}
	#[inline]
	pub fn mark_dirty(&self, id: VxWidgetId, dirty_flag: VxDirtyFlag) {
		(self.handler)(id, dirty_flag);
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum VxSpatialHierarchyFlag {
	#[default]
	Flat,
	HierarchyParent,
	HierarchyChild,
}

pub struct VxWidgetContext {
	id: Option<VxWidgetId>,
	transform: VxTransform,
	visible: bool,
	z_value: i32,
	bounding_rect: VxRect,
	dirty_command_sender: Option<VxDirtyCommandSender>,
	block_dirty: bool,
	block_signal: bool,
	is_enabled: bool,
	parent: Option<VxWidgetId>,
	children: Vec<VxWidgetId>,
	layout: VxSpatialLayout,
	spatial_hierarchy_flag: VxSpatialHierarchyFlag,
	spatial_hierarchy_parent: VxWidgetId,
	update_mode: VxRenderMode,

	children_widgets: Vec<Box<dyn VxWidget>>,
}

impl VxWidgetContext {
	pub fn new(parent: Option<VxWidgetId>) -> Self {
		Self {
			id: None,
			transform: VxTransform::default(),
			visible: true,
			z_value: 0,
			bounding_rect: VxRect::default(),
			dirty_command_sender: None,
			block_dirty: false,
			block_signal: false,
			is_enabled: true,
			parent,
			children: Vec::new(),
			layout: VxSpatialLayout::new(),
			spatial_hierarchy_flag: VxSpatialHierarchyFlag::Flat,
			spatial_hierarchy_parent: VxWidgetId::new_invalid(),
			update_mode: VxRenderMode::Retained,
			children_widgets: Vec::new(),
		}
	}
	// getters
	#[inline]
	pub const fn widget_id(&self) -> Option<VxWidgetId> { self.id }
	#[inline]
	pub const fn parent(&self) -> Option<VxWidgetId> { self.parent }
	#[inline]
	pub const fn children(&self) -> &Vec<VxWidgetId> { &self.children }
	#[inline]
	pub const fn is_visible(&self) -> bool { self.visible }
	#[inline]
	pub const fn z_value(&self) -> i32 { self.z_value }
	#[inline]
	pub const fn bounding_rect(&self) -> VxRect { self.bounding_rect }
	#[inline]
	pub const fn layout(&self) -> &VxSpatialLayout { &self.layout }
	#[inline]
	pub const fn layout_mut(&mut self) ->&mut VxSpatialLayout { &mut self.layout }

	#[inline]
	pub(crate) const fn spatial_hierarchy_parent(&self) -> VxWidgetId {
		self.spatial_hierarchy_parent
	}
	#[inline]
	pub(crate) fn take_children_widgets(&mut self) -> Vec<Box<dyn VxWidget>> {
		std::mem::take(&mut self.children_widgets)
	}

	// setters
	#[inline]
	pub(crate) const fn set_widget_id(&mut self, id: VxWidgetId) { self.id = Some(id); }
	#[inline]
	pub(crate) fn set_dirty_command_sender(&mut self, sender: VxDirtyCommandSender) {
		self.dirty_command_sender = Some(sender);
	}
	#[inline]
	pub fn set_parent(&mut self, parent: VxWidgetId) {
		if self.parent == Some(parent) { return; }
		self.parent = Some(parent);
	}
	#[inline]
	pub fn set_visible(&mut self, visible: bool) {
		if self.visible == visible { return; }
		self.visible = visible;
		self.set_dirty_flag(VxDirtyFlag::REPAINT);
	}
	#[inline]
	pub fn set_z_value(&mut self, z: i32) {
		if self.z_value == z { return; }
		self.z_value = z;
		self.set_dirty_flag(VxDirtyFlag::REPAINT);
	}
	#[inline]
	pub fn set_bounding_rect(&mut self, rect: VxRect) {
		if self.bounding_rect == rect { return; }
		self.bounding_rect = rect.with_pos(VxVec2::default());
		self.set_dirty_flag(VxDirtyFlag::LAYOUT);
	}
	#[inline]
	pub fn set_layout(&mut self, layout: VxSpatialLayout) {
		self.layout = layout;
		self.set_dirty_flag(VxDirtyFlag::LAYOUT);
	}
	#[inline]
	pub(crate) fn set_spatial_hierarchy_parent(&mut self, spatial_hierarchy_parent: VxWidgetId) {
		if self.spatial_hierarchy_parent == spatial_hierarchy_parent { return; }
		self.spatial_hierarchy_parent = spatial_hierarchy_parent;
		self.set_dirty_flag(VxDirtyFlag::LAYOUT);
	}
	#[inline]
	pub fn add_child(&mut self, child: VxWidgetId) { self.children.push(child); }
	#[inline]
	pub fn add_child_widget<W: VxWidget>(&mut self, child: W) {
		self.add_child_widget_box(child.into_box());
	}
	#[inline]
	pub fn add_child_widget_box(&mut self, child: Box<dyn VxWidget>) {
		self.children_widgets.push(child);
	}
}

// geometries
impl VxWidgetContext {
	#[inline]
	pub const fn pos(&self) -> VxVec2 { self.transform().pos() }
	#[inline]
	pub const fn angle(&self) -> VxAngle { self.transform().rotation() }
	#[inline]
	pub const fn scale(&self) -> VxSize { self.transform().scale() }
	#[inline]
	pub const fn center_pivot(&self) -> VxVec2 { self.transform().pivot() }
	#[inline]
	pub const fn transform(&self) -> VxTransform { self.transform }
	#[inline]
	pub fn set_pos(&mut self, pos: VxVec2) {
		if self.transform.pos() == pos { return; }
		self.transform.set_pos(pos);
		self.set_dirty_flag(VxDirtyFlag::LAYOUT);
	}
	#[inline]
	pub fn set_angle(&mut self, angle: VxAngle) {
		if self.transform.rotation() == angle { return; }
		self.transform.set_rotation(angle);
		self.set_dirty_flag(VxDirtyFlag::LAYOUT);
	}
	#[inline]
	pub fn set_scale(&mut self, scale: VxSize) {
		if self.transform.scale() == scale { return; }
		self.transform.set_scale(scale);
		self.set_dirty_flag(VxDirtyFlag::LAYOUT);
	}
	#[inline]
	pub fn set_center_pivot(&mut self, pivot: VxVec2) {
		if self.transform.pivot() == pivot { return; }
		self.transform.set_center_pivot(pivot);
		self.set_dirty_flag(VxDirtyFlag::LAYOUT);
	}
	#[inline]
	pub fn set_transform(&mut self, transform: VxTransform) {
		if self.transform == transform { return; }
		self.transform = transform;
		self.set_dirty_flag(VxDirtyFlag::LAYOUT);
	}
}

// flags
impl VxWidgetContext {
	#[inline]
	pub const fn is_block_dirty(&self) -> bool { self.block_dirty }
	#[inline]
	pub const fn is_block_signal(&self) -> bool { self.block_signal }
	#[inline]
	pub const fn is_enabled(&self) -> bool { self.is_enabled }
	#[inline]
	pub const fn spatial_hierarchy_flag(&self) -> VxSpatialHierarchyFlag {
		self.spatial_hierarchy_flag
	}
	#[inline]
	pub const fn update_mode(&self) -> VxRenderMode { self.update_mode }

	#[inline]
	pub fn set_block_dirty(&mut self, block_dirty: bool) {
		self.block_dirty = block_dirty;
	}
	#[inline]
	pub fn set_block_signal(&mut self, block_signal: bool) {
		self.block_signal = block_signal;
	}
	#[inline]
	pub fn set_enabled(&mut self, enabled: bool) {
		self.is_enabled = enabled;
	}
	#[inline]
	pub fn set_dirty_flag(&self, dirty: VxDirtyFlag) {
		if self.block_dirty { return; }
		if let (Some(sender), Some(id)) = (&self.dirty_command_sender, self.id) {
			sender.mark_dirty(id, dirty);
		}
	}
	#[inline]
	pub fn set_spatial_hierarchy_flag(&mut self, spatial_hierarchy_flag: VxSpatialHierarchyFlag) {
		if self.spatial_hierarchy_flag == spatial_hierarchy_flag { return; }
		self.spatial_hierarchy_flag = spatial_hierarchy_flag;
		self.set_dirty_flag(VxDirtyFlag::REBUILD_ALL);
	}
	#[inline]
	pub const fn set_update_mode(&mut self, update_mode: VxRenderMode) {
		self.update_mode = update_mode;
	}
}

pub trait VxWidgetAccessor: std::any::Any {
	fn context(&self) -> &VxWidgetContext;
	fn context_mut(&mut self) -> &mut VxWidgetContext;
	fn as_any(&self) -> &dyn std::any::Any;
	fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
	fn into_any(self: Box<Self>) -> Box<dyn std::any::Any>;
}

pub trait VxWidget: VxWidgetAccessor {
	#[inline]
	fn bounding_rect(&self) -> VxRect {
		self.context().bounding_rect()
	}
	fn paint(&mut self, painter: &mut VxPainter, palette: &VxColorPalette);
	fn immediate_paint(&mut self, input: &VxInputState, painter: &mut VxPainter) {
		let _ = input;
		let _ = painter;
	}
	fn size_hint(&mut self, creator: &mut VxBoundingRectCreator) -> VxSizeHint;

	#[inline]
	fn as_box_layout(&self) -> Option<&dyn VxBoxLayout> { None }
	#[inline]
	fn as_box_layout_mut(&mut self) -> Option<&mut dyn VxBoxLayout> { None }

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

pub trait VxWidgetContextWrapExt: VxWidget {
	#[inline]
	fn widget_id(&self) -> Option<VxWidgetId> { self.context().widget_id() }
	#[inline]
	fn parent(&self) -> Option<VxWidgetId> { self.context().parent() }
	#[inline]
	fn children(&self) -> &Vec<VxWidgetId> { &self.context().children() }
	#[inline]
	fn is_visible(&self) -> bool { self.context().is_visible() }
	#[inline]
	fn z_value(&self) -> i32 { self.context().z_value() }
	#[inline]
	fn spatial_hierarchy_flag(&self) -> VxSpatialHierarchyFlag { self.context().spatial_hierarchy_flag() }
	#[inline]
	fn update_mode(&self) -> VxRenderMode { self.context().update_mode() }
	#[inline]
	fn is_block_dirty(&self) -> bool { self.context().is_block_dirty() }
	#[inline]
	fn is_block_signal(&self) -> bool { self.context().is_block_signal() }
	#[inline]
	fn is_enabled(&self) -> bool { self.context().is_enabled() }

	#[inline]
	fn set_parent(&mut self, parent: VxWidgetId) { self.context_mut().set_parent(parent); }
	#[inline]
	fn set_visible(&mut self, visible: bool) { self.context_mut().set_visible(visible); }
	#[inline]
	fn set_z_value(&mut self, z: i32) { self.context_mut().set_z_value(z); }
	#[inline]
	fn set_dirty_flag(&mut self, dirty: VxDirtyFlag) { self.context_mut().set_dirty_flag(dirty); }
	#[inline]
	fn set_block_signal(&mut self, block_signal: bool) { self.context_mut().set_block_signal(block_signal); }
	#[inline]
	fn set_enabled(&mut self, enabled: bool) { self.context_mut().set_enabled(enabled); }
	#[inline]
	fn set_spatial_hierarchy_flag(&mut self, spatial_hierarchy_flag: VxSpatialHierarchyFlag) {
		self.context_mut().set_spatial_hierarchy_flag(spatial_hierarchy_flag);
	}
	#[inline]
	fn set_update_mode(&mut self, update_mode: VxRenderMode) {
		self.context_mut().set_update_mode(update_mode);
	}
	#[inline]
	fn set_block_dirty(&mut self, block: bool) {
		self.context_mut().set_block_dirty(block);
	}

	fn remove_child(&mut self, child_id: VxWidgetId) {
		self.context_mut().children.retain(|&id| id != child_id);
	}
}

pub trait VxWidgetGeometryExt: VxWidget {
	#[inline]
	fn pos(&self) -> VxVec2 { self.context().pos() }
	#[inline]
	fn angle(&self) -> VxAngle { self.context().angle() }
	#[inline]
	fn scale(&self) -> VxSize { self.context().scale() }
	#[inline]
	fn transform(&self) -> VxTransform { self.context().transform() }

	#[inline]
	fn set_pos(&mut self, pos: VxVec2) { self.context_mut().set_pos(pos); }
	#[inline]
	fn set_angle(&mut self, angle: VxAngle) { self.context_mut().set_angle(angle); }
	#[inline]
	fn set_scale(&mut self, scale: VxSize) { self.context_mut().set_scale(scale); }
	#[inline]
	fn set_center_pivot(&mut self, pivot: VxVec2) { self.context_mut().set_center_pivot(pivot); }
	#[inline]
	fn set_transform(&mut self, transform: VxTransform) { self.context_mut().set_transform(transform); }
}

pub trait VxWidgetLayoutExt: VxWidget {
	#[inline]
	fn layout(&self) -> &VxSpatialLayout { self.context().layout() }
	#[inline]
	fn layout_mut(&mut self) -> &mut VxSpatialLayout { self.context_mut().layout_mut() }
	#[inline]
	fn set_layout(&mut self, layout: VxSpatialLayout) {
		self.context_mut().set_layout(layout);
	}
}

pub trait VxIntoVxWidgetBox: VxWidget {
	fn into_box(self) -> Box<dyn VxWidget>
	where Self: Sized {
		Box::new(self)
	}
}

impl<T: VxWidget + ?Sized> VxWidgetContextWrapExt for T {}
impl<T: VxWidget + ?Sized> VxWidgetGeometryExt for T {}
impl<T: VxWidget + ?Sized> VxWidgetLayoutExt for T {}
impl<T: VxWidget + ?Sized> VxIntoVxWidgetBox for T {}

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
			pub pressed: $crate::abstracts::abstract_widgets::VxPressedSignal<Sender>,
			pub released: $crate::abstracts::abstract_widgets::VxReleasedSignal<Sender>,
			pub hovered: $crate::abstracts::abstract_widgets::VxHoveredSignal<Sender>,
			pub leaved: $crate::abstracts::abstract_widgets::VxLeavedSignal<Sender>,
			pub moved: $crate::abstracts::abstract_widgets::VxMovedSignal<Sender>,
			$( pub $field : $sig_name<Sender>, )*
			_marker: std::marker::PhantomData<Sender>,
		}

		impl<Sender: ?Sized> $name<Sender> {
			#[inline]
			pub fn new() -> Self {
				Self {
					pressed: $crate::abstracts::abstract_widgets::VxPressedSignal::new(),
					released: $crate::abstracts::abstract_widgets::VxReleasedSignal::new(),
					hovered: $crate::abstracts::abstract_widgets::VxHoveredSignal::new(),
					leaved: $crate::abstracts::abstract_widgets::VxLeavedSignal::new(),
					moved: $crate::abstracts::abstract_widgets::VxMovedSignal::new(),
					$( $field : $sig_name::new(), )*
					_marker: std::marker::PhantomData,
				}
			}
		}
	};
}

vx_widget_signals!(pub struct VxDefaultWidgetSignals {});