use std::{cell::RefCell, rc::Rc};

use ahash::{AHashMap, AHashSet};

use crate::{
	abstracts::{
		abstract_layouts::{
			VxBoundingRectCreator, VxBoxLayoutResolver, VxSpatialLayoutResolver
		}, abstract_widgets::*,
	}, core::{
		resource::VxAppResource, spatial_index::VxSpatialIndex
	}, painter::painter::VxPainter, types::{
		event::{
			VxEventResult,
			VxKeyEvent,
			VxMouseEvent
		}, gen_vector::{
			VxGenIndexWrapper, VxGenVector
		}, geometry::{
			VxRect,
			VxSize,
			VxVec2
		}, input::VxInputState, render_commands::{
			VxDirtyCheckResult,
			VxRenderMode
		}, style::VxColorPalette
	},
};

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum VxSpatialUpdateFlag<T> {
	Flat,
	Hierarchical(T),
}

struct VxDirtyProcessContext<'a> {
	widgets: &'a mut VxGenVector<Box<dyn VxWidget>>,
	bounding_rect_creator: &'a mut VxBoundingRectCreator<'a>,
	window_size: VxSize,
	spatial_layout_resolver: &'a mut VxSpatialLayoutResolver,
	box_layout_resolver: &'a mut VxBoxLayoutResolver,
	hbvh: &'a mut VxHbvh<VxWidgetId>,
}
impl<'a> VxDirtyProcessContext<'a> {
	#[inline]
	fn new(
		widgets: &'a mut VxGenVector<Box<dyn VxWidget>>,
		bounding_rect_creator: &'a mut VxBoundingRectCreator<'a>,
		window_size: VxSize,
		spatial_layout_resolver: &'a mut VxSpatialLayoutResolver,
		box_layout_resolver: &'a mut VxBoxLayoutResolver,
		hbvh: &'a mut VxHbvh<VxWidgetId>
	) -> Self {
		Self { widgets, bounding_rect_creator, window_size, spatial_layout_resolver, box_layout_resolver, hbvh }
	}

	pub fn process_update(&mut self, id: VxWidgetId, flag: VxDirtyFlag) -> VxDirtyCheckResult {
		let (is_immediate, children) = if let Some(w) = self.widgets.get(id) {
			let is_immediate = w.context().update_mode() == VxRenderMode::Immediate;
			let children = if w.children().is_empty() { None } else { Some(w.children().clone()) };
			(is_immediate, children)
		} else { (false, None) };
		if flag == VxDirtyFlag::CLEAN {
			return  if is_immediate { VxDirtyCheckResult::OnlyImmediate } else { VxDirtyCheckResult::None };
		}
		if flag.contains(VxDirtyFlag::LAYOUT) {
			self.process_update_layout(id);
		}
		if let Some(child) = children {
			for c in child {
				return self.process_update(c, flag);
			}
		}
		VxDirtyCheckResult::All
	}

	fn process_update_layout(&mut self, id: VxWidgetId) {
		let target_widget = Self::find_top_level_box_layout(self.widgets, id).unwrap_or_else(|| id);
		let initial_rect = if let Some(widget) = self.widgets.get(target_widget) {
			if widget.parent().is_none() {
				VxRect::from_pos_size((0, 0).into(), self.window_size)
			} else {
				widget.bounding_rect().with_pos(widget.pos())
			}
		} else {
			return;
		};
		
		self.box_layout_resolver.clear_cache();
		self.box_layout_resolver.resolve_container(
			self.bounding_rect_creator,
			target_widget,
			initial_rect,
			self.widgets,
		);

		let computed_rects = self.box_layout_resolver.take_computed_rects();

		for (update_id, rect) in computed_rects {
			if let Some(widget) = self.widgets.get_mut(update_id) {
				let stats = widget.context_mut();
				stats.set_block_dirty(true);
				stats.set_pos(rect.pos());
				stats.set_bounding_rect(rect.with_pos(VxVec2::default()));
				stats.set_block_dirty(false);

				self.process_update_spatial_index(update_id, rect);
			}
		}
	}

	fn process_update_spatial_index(&mut self, id: VxWidgetId, rect: VxRect) {
		let Some((spatial_flag, spatial_parent_id)) = self.widgets.get(id)
			.map(|w| (w.spatial_hierarchy_flag(), w.context().spatial_hierarchy_parent())) else { return; };

		let global_pos = Self::calc_global_pos(self.widgets, id);
		let global_rect = rect.with_pos(global_pos);

		match spatial_flag {
			VxSpatialHierarchyFlag::HierarchyChild => {
				if let Some(index) = self.hbvh.hierarchical.get_mut(&spatial_parent_id) {
					index.update_at(id, rect);
					self.hbvh.need_update_index.insert(VxSpatialUpdateFlag::Hierarchical(spatial_parent_id));
				}
			}
			_ => {
				self.hbvh.flat.update_at(id, global_rect);
				self.hbvh.need_update_index.insert(VxSpatialUpdateFlag::Flat);
			}
		}
	}

	fn find_top_level_box_layout(widgets: &VxGenVector<Box<dyn VxWidget>>, mut current_id: VxWidgetId) -> Option<VxWidgetId> {
		let mut last_top_layout = None;
		while let Some(widget) = widgets.get(current_id) {
			if widget.as_box_layout().is_some() {
				last_top_layout = Some(current_id);
			}
			if let Some(parent) = widget.parent() {
				current_id = parent;
			} else {
				break;
			}
		}
		last_top_layout
	}

	pub fn finish(&mut self) {
		for flag in self.hbvh.need_update_index.drain() {
			match flag {
				VxSpatialUpdateFlag::Flat => self.hbvh.flat.optimize(),
				VxSpatialUpdateFlag::Hierarchical(id) => {
					if let Some(index) = self.hbvh.hierarchical.get_mut(&id) {
						index.optimize();
					}
				}
			}
		}
	}

	pub fn calc_global_pos(widgets: &VxGenVector<Box<dyn VxWidget>>, mut id: VxWidgetId) -> VxVec2 {
		let mut global_pos = VxVec2::default();
		while let Some(widget) = widgets.get(id) {
			global_pos += widget.pos();
			if let Some(parent_id) = widget.parent() {
				id = parent_id;
			} else {
				break;
			}
		}
		global_pos
	}
}

pub struct VxHbvh<T: VxGenIndexWrapper> {
	pub flat: VxSpatialIndex<T>,
	pub hierarchical: AHashMap<T, VxSpatialIndex<T>>,
	pub need_update_index: AHashSet<VxSpatialUpdateFlag<T>>,
}
impl<T: VxGenIndexWrapper> VxHbvh<T> {
	#[inline]
	pub fn new() -> Self {
		Self {
			flat: VxSpatialIndex::new(),
			hierarchical: AHashMap::new(),
			need_update_index: AHashSet::new(),
		}
	}
	#[inline]
	pub fn add_hierarchical(&mut self, id: T) {
		self.hierarchical.insert(id, VxSpatialIndex::new());
	}
	#[inline]
	pub fn traverse_flat(&self, pos: VxVec2) -> Vec<T> {
		self.flat.hit_test(pos)
	}
	#[inline]
	pub fn traverse_hierarchical(&self, id: T, pos: VxVec2) -> Option<Vec<T>> {
		Some(self.hierarchical.get(&id)?.hit_test(pos))
	}
}

pub struct VxScene {
	widgets: VxGenVector<Box<dyn VxWidget>>,
	top_level_widgets: Vec<VxWidgetId>,
	immediate_widgets: Vec<VxWidgetId>,
	current_selected_widgets: Option<VxWidgetId>,
	current_hovered_widgets: Option<VxWidgetId>,

	dirty_command_sender: VxDirtyCommandSender,
	dirty_queue: Rc<RefCell<AHashMap<VxWidgetId, VxDirtyFlag>>>,

	hbvh: VxHbvh<VxWidgetId>,

	spatial_layout_resolver: VxSpatialLayoutResolver,
	box_layout_resolver: VxBoxLayoutResolver,
}

impl VxScene {
	pub fn new() -> Self {
		let dirty_queue = Rc::new(RefCell::new(AHashMap::new()));
		let queue_clone = dirty_queue.clone();
		Self {
			widgets: VxGenVector::new(),
			top_level_widgets: Vec::new(),
			immediate_widgets: Vec::new(),
			current_selected_widgets: None,
			current_hovered_widgets: None,
			dirty_command_sender: VxDirtyCommandSender::new(move |id, flag| {
				*queue_clone.borrow_mut().entry(id).or_insert(flag) |= flag;
			}),
			dirty_queue,
			hbvh: VxHbvh::new(),
			spatial_layout_resolver: VxSpatialLayoutResolver::new(),
			box_layout_resolver: VxBoxLayoutResolver::new(),
		}
	}

	fn paint_widget(
		widgets: &mut VxGenVector<Box<dyn VxWidget>>,
		res: &mut VxAppResource,
		painter: &mut VxPainter,
		palette: &VxColorPalette,
		id: VxWidgetId,
	) {
		let Some(widget) = widgets.get_mut(id) else { return; };
		if !widget.is_visible() {
			return;
		}
		
		painter.push_transform(widget.transform());
		widget.paint(painter, palette);
		painter.set_vertex_z_value(widget.z_value());

		for child in widget.children().clone() {
			Self::paint_widget(widgets, res, painter, palette, child);
		}
		painter.pop_transform();
	}
	fn immediate_paint_widget(
		widgets: &mut VxGenVector<Box<dyn VxWidget>>,
		res: &mut VxAppResource,
		input: &VxInputState,
		painter: &mut VxPainter,
		id: VxWidgetId,
	) {
		let Some(widget) = widgets.get_mut(id) else { return; };
		if !widget.is_visible() {
			return;
		}

		widget.immediate_paint(input, painter);
		painter.set_vertex_z_value(widget.z_value());

		for child in widget.children().clone() {
			Self::immediate_paint_widget(widgets, res, input, painter, child);
		}
	}

	pub(crate) fn check_dirty(&mut self, res: &mut VxAppResource, window_size: VxSize) -> VxDirtyCheckResult {
		let mut result = VxDirtyCheckResult::None;
		let mut bounding_rect_creator = VxBoundingRectCreator::new(res);
		let queue = self.dirty_queue.take();
		let mut process_ctx = VxDirtyProcessContext::new(
			&mut self.widgets, &mut bounding_rect_creator, window_size,
			&mut self.spatial_layout_resolver, &mut self.box_layout_resolver, &mut self.hbvh,
		);
		for (id, flag) in queue.into_iter() {
			result.merge(process_ctx.process_update(id, flag));
		}
		if result == VxDirtyCheckResult::None && !self.immediate_widgets.is_empty() {
			result = VxDirtyCheckResult::OnlyImmediate;
		}
		process_ctx.finish();
		result
	}
	pub fn add_widget<W: VxWidget>(&mut self, widget: W) -> VxWidgetHandler<W> {
		VxWidgetHandler::<W>::new(self.add_widget_box(widget.into_box()))
	}

	pub fn add_widget_box(&mut self, mut widget: Box<dyn VxWidget>) -> VxWidgetId {
		let children = widget.context_mut().take_children_widgets();
		let id = VxWidgetId::new(self.widgets.vacant_id());

		Self::register_widget(
			&mut widget,
			id,
			self.dirty_command_sender.clone(),
			&mut self.top_level_widgets,
			&mut self.immediate_widgets,
			&mut self.hbvh,
		);

		let parent_hierarchy_flag = widget.spatial_hierarchy_flag();
		self.widgets.insert(widget);
		
		for mut child in children {
			child.set_parent(id);
			match parent_hierarchy_flag {
				VxSpatialHierarchyFlag::HierarchyParent => {
					child.set_spatial_hierarchy_flag(VxSpatialHierarchyFlag::HierarchyChild);
					child.context_mut().set_spatial_hierarchy_parent(id);
				}
				VxSpatialHierarchyFlag::HierarchyChild => {
					child.set_spatial_hierarchy_flag(VxSpatialHierarchyFlag::HierarchyChild);
					child.context_mut().set_spatial_hierarchy_parent(Self::find_spatial_index(&self.widgets, id));
				}
				_ => {}
			}
			let child_id = self.add_widget_box(child);
			if let Some(parent) = self.widgets.get_mut(id) {
				parent.context_mut().add_child(child_id);
			}
		}
		self.dirty_command_sender.mark_dirty(id, VxDirtyFlag::REBUILD_ALL);
		id
	}

	fn register_widget<W: VxWidget + ?Sized>(
		widget: &mut Box<W>,
		widget_id: VxWidgetId,
		dirty_command_sender: VxDirtyCommandSender,
		top_level_widgets: &mut Vec<VxWidgetId>,
		immediate_widgets: &mut Vec<VxWidgetId>,
		hbvh: &mut VxHbvh<VxWidgetId>,
	) {
		widget.context_mut().set_widget_id(widget_id);
		widget.context_mut().set_dirty_command_sender(dirty_command_sender);

		if widget.parent().is_none() {
			top_level_widgets.push(widget_id);
		}
		if widget.update_mode() == VxRenderMode::Immediate {
			immediate_widgets.push(widget_id);
		}

		// HBVHの準備
		match widget.spatial_hierarchy_flag() {
			VxSpatialHierarchyFlag::HierarchyParent => {
				hbvh.add_hierarchical(widget_id);
			}
			_ => {}
		}
	}

	fn find_spatial_index(widgets: &VxGenVector<Box<dyn VxWidget>>, mut current_id: VxWidgetId) -> VxWidgetId {
		while let Some(parent_id) = widgets.get(current_id).and_then(|w| w.parent()) {
			if let Some(parent) = widgets.get(parent_id) {
				if parent.spatial_hierarchy_flag() == VxSpatialHierarchyFlag::HierarchyParent {
					return parent_id;
				}
				current_id = parent_id;
			}
		}
		current_id
	}

	#[inline]
	fn find_widget_at(&self, pos: VxVec2) -> Option<VxWidgetId> {
		self.traverse_widget_at(&self.hbvh.flat, pos)
	}

	fn traverse_widget_at(&self, target_index: &VxSpatialIndex<VxWidgetId>, pos: VxVec2) -> Option<VxWidgetId> {
		let res = target_index.hit_test(pos);
		if res.is_empty() { return None; }

		let traverse_result = res.into_iter()
			.filter_map(|id| {
				let widget = self.widgets.get(id)?;
				let global_pos = VxDirtyProcessContext::calc_global_pos(&self.widgets, id);
				let global_rect = widget.bounding_rect().with_pos(global_pos);
				if widget.is_visible() && global_rect.contains(pos) {
					let result = VxSpatialTraverseResult::new(
						id, widget.spatial_hierarchy_flag()
					);
					Some((widget.z_value(), result))
				} else {
					None
				}
			})
			.max_by_key(|(z, _)| *z)
			.map(|(_, result)| result)?;

		let (res_id, flag) = traverse_result.decompose();

		match flag {
			VxSpatialHierarchyFlag::HierarchyParent => {
				let target_index = self.hbvh.hierarchical.get(&res_id)?;
				let global_pos = VxDirtyProcessContext::calc_global_pos(&self.widgets, res_id);
				return self.traverse_widget_at(target_index, global_pos);
			}
			_ => {}
		}

		Some(res_id)
	}
}

impl VxScene {
	pub fn has_immediate_widget(&self) -> bool {
		!self.immediate_widgets.is_empty()
	}
	pub fn get_widget<W: VxWidget>(&self, handler: VxWidgetHandler<W>) -> Option<&W> {
		self.widgets.get(handler.id())?
			.as_any()
			.downcast_ref::<W>()
	}
	pub fn get_widget_mut<W: VxWidget>(&mut self, handler: VxWidgetHandler<W>) -> Option<&mut W> {
		self.widgets.get_mut(handler.id())?
			.as_any_mut()
			.downcast_mut::<W>()
	}
	pub fn remove_widget<W: VxWidget>(&mut self, handler: VxWidgetHandler<W>) -> Option<W> {
		let id = handler.id();
		let widget = self.remove_widget_id(id)?;
		let widget_downcast = widget.into_any().downcast().ok()?;
		Some(*widget_downcast)
	}
	pub fn remove_widget_id(&mut self, id: VxWidgetId) -> Option<Box<dyn VxWidget>> {
		let widget = self.widgets.remove(id)?;
		self.top_level_widgets.retain(|&top_id| top_id != id);
		self.immediate_widgets.retain(|&imm_id| imm_id != id);
		self.current_hovered_widgets = self.current_hovered_widgets.filter(|&i| i == id);
		self.current_selected_widgets = self.current_selected_widgets.filter(|&i| i == id);
		self.hbvh.flat.remove(id);
		if let Some(spatial_index) = self.hbvh.hierarchical.get_mut(&id) {
			spatial_index.remove(id);
		}
		for &child in widget.children() {
			self.remove_widget_id(child);
		}
		if let Some(parent_id) = widget.parent() {
			if let Some(parent) = self.widgets.get_mut(parent_id) {
				parent.remove_child(id);
			}
		}
		Some(widget)
	}
}

impl VxScene {
	pub fn paint_event(&mut self, res: &mut VxAppResource, painter: &mut VxPainter, palette: &VxColorPalette) {
		self.top_level_widgets.iter().for_each(|id| {
			Self::paint_widget(&mut self.widgets, res, painter, palette, *id);
		});
	}
	pub fn immediate_paint_event(&mut self, res: &mut VxAppResource, input: &VxInputState, painter: &mut VxPainter) {
		self.immediate_widgets.iter().for_each(|id| {
			Self::immediate_paint_widget(&mut self.widgets, res, input, painter, *id);
		});
	}

	fn send_event_to_widget<E>(
		&mut self,
		mut start_id: Option<VxWidgetId>,
		event: &E,
		handler: impl Fn(&mut Box<dyn VxWidget>, &E) -> VxEventResult
	) -> VxEventResult {
		while let Some(id) = start_id {
			let Some(widget) = self.widgets.get_mut(id) else { break; };
			let result = handler(widget, event);
			if result == VxEventResult::Accept {
				return VxEventResult::Accept;
			}
			start_id = widget.parent();
		}
		VxEventResult::Ignore
	}
	pub fn mouse_press_event(&mut self, event: &VxMouseEvent)  -> VxEventResult {
		let current_id = self.find_widget_at(event.pos());
		self.current_selected_widgets = current_id;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e | w.mouse_press_event(e)
		)
	}
	pub fn mouse_release_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let current_id = self.find_widget_at(event.pos());
		self.current_selected_widgets = current_id;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.mouse_release_event(e)
		)
	}
	pub fn mouse_move_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let new_hover_id = self.find_widget_at(event.pos());
		let old_hover_id = self.current_hovered_widgets;

		if new_hover_id != old_hover_id {
			if let Some(old_id) = old_hover_id {
				self.send_event_to_widget(
					Some(old_id),
					event,
					|w, e| w.mouse_leave_event(e)
				);
			}
			if let Some(new_id) = new_hover_id {
				self.send_event_to_widget(
					Some(new_id),
					event,
					|w, e| w.mouse_enter_event(e)
				);
			}
			self.current_hovered_widgets = new_hover_id;
		}

		self.send_event_to_widget(
			new_hover_id,
			event,
			|w, e| w.mouse_move_event(e)
		)
	}
	pub fn mouse_wheel_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let current_id = self.find_widget_at(event.pos());
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.mouse_wheel_event(e)
		)
	}
	// KeyboardEvents
	pub fn key_press_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		let current_id = self.current_selected_widgets;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.key_press_event(e)
		)
	}
	pub fn key_release_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		let current_id = self.current_selected_widgets;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.key_release_event(e)
		)
	}

	pub fn resized_event(&mut self, _: VxSize) -> VxEventResult {
		self.top_level_widgets.iter().for_each(|id| {
			if let Some(widget) = self.widgets.get(*id) {
				widget.context().set_dirty_flag(VxDirtyFlag::LAYOUT);
			}
		});
		VxEventResult::Accept
	}
}

#[derive(Clone, Copy, Debug)]
struct VxSpatialTraverseResult {
	id: VxWidgetId,
	flag: VxSpatialHierarchyFlag,
}

impl VxSpatialTraverseResult {
	#[inline]
	pub fn new(id: VxWidgetId, flag: VxSpatialHierarchyFlag) -> Self {
		Self { id, flag }
	}
	#[inline]
	pub fn decompose(self) -> (VxWidgetId, VxSpatialHierarchyFlag) {
		(self.id, self.flag)
	}
}