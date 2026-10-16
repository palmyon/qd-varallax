use std::{cell::RefCell, rc::Rc};

use ahash::{AHashMap, AHashSet};

use crate::{
	abstracts::{
		abstract_layouts::{
			VxBoundingRectCreator, VxBoxLayoutResolver
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

#[derive(Clone, Copy, Debug)]
struct VxSpatialTraverseResult {
	id: VxWidgetId,
	flag: VxHbvhHierarchy,
}

impl VxSpatialTraverseResult {
	#[inline]
	pub fn new(id: VxWidgetId, flag: VxHbvhHierarchy) -> Self {
		Self { id, flag }
	}
	#[inline]
	pub fn decompose(self) -> (VxWidgetId, VxHbvhHierarchy) {
		(self.id, self.flag)
	}
}

struct VxSelectWidgetContainer {
	selected_single: Option<VxWidgetId>,
	selected: AHashSet<VxWidgetId>,
	hovered: Option<VxWidgetId>,
}
impl VxSelectWidgetContainer {
	pub fn new() -> Self {
		Self {
			selected_single: None,
			selected: AHashSet::new(),
			hovered: None,
		}
	}
	#[inline]
	pub fn selected_single(&self) -> Option<VxWidgetId> { self.selected_single }
	#[inline]
	pub fn selected(&self) -> &AHashSet<VxWidgetId> { &self.selected }
	#[inline]
	pub fn hovered(&self) -> Option<VxWidgetId> { self.hovered }
	#[inline]
	pub fn set_selected_single(&mut self, selected: Option<VxWidgetId>) {
		self.selected_single = selected;
	}
	#[inline]
	pub fn add_selected(&mut self, selected: VxWidgetId) {
		self.selected.insert(selected);
	}
	#[inline]
	pub fn remove_selected(&mut self, id: VxWidgetId) {
		self.selected.remove(&id);
	}
	#[inline]
	pub fn set_hovered(&mut self, hovered: Option<VxWidgetId>) {
		self.hovered = hovered;
	}

	#[inline]
	pub fn clear_selected(&mut self) {
		self.selected.clear();
	}
	#[inline]
	pub fn take_selected(&mut self) -> AHashSet<VxWidgetId> {
		std::mem::take(&mut self.selected)
	}
}

pub struct VxScene {
	widgets: VxGenVector<Box<dyn VxWidget>>,
	top_level_widgets: Vec<VxWidgetId>,
	immediate_widgets: Vec<VxWidgetId>,

	dirty_command_sender: VxDirtyCommandSender,
	dirty_queue: Rc<RefCell<AHashMap<VxWidgetId, VxDirtyFlag>>>,

	select_container: VxSelectWidgetContainer,
	hbvh: VxHbvh<VxWidgetId>,

	box_layout_resolver: VxBoxLayoutResolver,
	window_size: VxSize,
}

impl VxScene {
	pub fn new() -> Self {
		let queue = Rc::new(RefCell::new(AHashMap::new()));
		let queue_clone = queue.clone();
		Self {
			widgets: VxGenVector::new(),
			top_level_widgets: Vec::new(),
			immediate_widgets: Vec::new(),
			dirty_command_sender: VxDirtyCommandSender::new(move |id, flag| {
				*queue_clone.borrow_mut().entry(id).or_insert(flag) |= flag;
			}),
			dirty_queue: queue,
			select_container: VxSelectWidgetContainer::new(),
			hbvh: VxHbvh::new(),
			box_layout_resolver: VxBoxLayoutResolver::new(),
			window_size: VxSize::default(),
		}
	}
}

impl VxScene {
	#[inline]
	pub fn has_immediate_widget(&self) -> bool {
		!self.immediate_widgets.is_empty()
	}
	#[inline]
	pub fn get_widget<W: VxWidget>(&self, handler: VxWidgetHandler<W>) -> Option<&W> {
		self.widgets.get(*handler)?.as_any().downcast_ref::<W>()
	}
	#[inline]
	pub fn get_widget_mut<W: VxWidget>(&mut self, handler: VxWidgetHandler<W>) -> Option<&mut W> {
		self.widgets.get_mut(*handler)?.as_any_mut().downcast_mut::<W>()
	}
	
	pub fn add_widget<W: VxWidget>(&mut self, widget: W) -> VxWidgetHandler<W> {
		VxWidgetHandler::new(self.process_add_widget(widget.into_box()))
	}
	pub fn add_widget_box(&mut self, widget: Box<dyn VxWidget>) -> VxWidgetId {
		self.process_add_widget(widget)
	}

	pub fn remove_widget<W: VxWidget>(&mut self, handler: VxWidgetHandler<W>) -> Option<W> {
		let widget = self.remove_widget_id(*handler)?;
		let w = widget.into_any().downcast::<W>().ok()?;
		Some(*w)
	}
	#[inline]
	pub fn remove_widget_id(&mut self, id: VxWidgetId) -> Option<Box<dyn VxWidget>> {
		self.process_remove_widget_id(id)
	}

	#[inline]
	pub fn find_widget_at(&self, at: VxVec2) -> Option<VxWidgetId> {
		self.traverse_widget_at(&self.hbvh.flat, at)
	}
}

impl VxScene {
	pub(crate) fn paint_event(&mut self, res: &mut VxAppResource, painter: &mut VxPainter, palette: &VxColorPalette) {
		self.top_level_widgets.iter().for_each(|id| {
			Self::process_paint_widget(&mut self.widgets, *id, res, painter, palette);
		});
	}
	pub(crate) fn immediate_paint_event(&mut self, input: &VxInputState, painter: &mut VxPainter) {
		self.immediate_widgets.iter().for_each(|id| {
			Self::process_immediate_paint_widget(&mut self.widgets, *id, painter, input);
		});
	}

	fn process_paint_widget(widgets: &mut VxGenVector<Box<dyn VxWidget>>, id: VxWidgetId, res: &mut VxAppResource, painter: &mut VxPainter, palette: &VxColorPalette) {
		let Some(widget) = widgets.get_mut(id) else { return; };
		if !widget.is_visible() {
			return;
		}

		painter.push_transform(widget.transform());
		painter.record_transform();
		widget.paint(painter, palette);
		painter.remove_stacked_transform();

		painter.set_vertex_z_value(widget.z_value());

		if !widget.children().is_empty() {
			for child in widget.children().clone() {
				Self::process_paint_widget(widgets, child, res, painter, palette);
			}
		}

		painter.pop_transform();
	}

	fn process_immediate_paint_widget(widgets: &mut VxGenVector<Box<dyn VxWidget>>, id: VxWidgetId, painter: &mut VxPainter, input: &VxInputState) {
		let Some(widget) = widgets.get_mut(id) else { return; };
		if !widget.is_visible() {
			return;
		}

		painter.push_transform(widget.transform());
		painter.record_transform();
		widget.immediate_paint(input, painter);
		painter.remove_stacked_transform();
		painter.pop_transform();

		painter.set_vertex_z_value(widget.z_value());
	}

	fn process_add_widget(&mut self, mut widget: Box<dyn VxWidget>) -> VxWidgetId {
		let id = VxWidgetId::new(self.widgets.vacant_id());

		widget.context_mut().set_widget_id(id);
		widget.context_mut().set_dirty_command_sender(self.dirty_command_sender.clone());
		widget.set_dirty(VxDirtyFlag::REBUILD_ALL);

		if widget.is_immediate() {
			self.immediate_widgets.push(id);
		}
		if widget.parent().is_none() {
			self.top_level_widgets.push(id);
		}
		if widget.hbvh_hierarchy() == VxHbvhHierarchy::Parent {
			self.hbvh.add_hierarchical(id);
		}

		if let Some(parent_id) = widget.parent() {
			if let Some(parent) = self.widgets.get(parent_id) {
				let hbvh_hierarchy = parent.hbvh_hierarchy();
				match hbvh_hierarchy {
					VxHbvhHierarchy::Parent => {
						widget.set_hbvh_hierarchy(VxHbvhHierarchy::Child);
						widget.context_mut().set_hbvh_hierarchy_parent(Some(parent_id));
					}
					VxHbvhHierarchy::Child => {
						if widget.hbvh_hierarchy() != VxHbvhHierarchy::Parent {
							widget.set_hbvh_hierarchy(VxHbvhHierarchy::Child);
						}
						widget.context_mut().set_hbvh_hierarchy_parent(parent.context().hbvh_hierarchy_parent());
					}
					_ => {}
				}
			}
		}

		self.widgets.insert(widget);
		let children = self.widgets.get_mut(id).unwrap().context_mut().take_children_widgets();

		for mut child in children {
			child.set_parent(id);
			let child_id = self.process_add_widget(child);
			self.widgets.get_mut(id).unwrap().add_child(child_id);
		}

		id
	}

	fn process_remove_widget_id(&mut self, id: VxWidgetId) -> Option<Box<dyn VxWidget>> {
		let widget = self.widgets.remove(id)?;
		self.immediate_widgets.retain(|i| *i != id);
		self.top_level_widgets.retain(|i| *i != id);

		if self.select_container.selected_single() == Some(id) {
			self.select_container.set_selected_single(None);
		}
		if self.select_container.hovered() == Some(id) {
			self.select_container.set_hovered(None);
		}
		if self.select_container.selected().contains(&id) {
			self.select_container.remove_selected(id);
		}

		for child in widget.children() {
			self.remove_widget_id(*child);
		}

		match widget.hbvh_hierarchy() {
			VxHbvhHierarchy::Child => {
				if let Some(hbvh_parent) = widget.context().hbvh_hierarchy_parent() {
					if let Some(bvh) = self.hbvh.hierarchical.get_mut(&hbvh_parent) {
						bvh.remove(id);
					}
				}
			}
			VxHbvhHierarchy::Parent => {
				self.hbvh.hierarchical.remove(&id);
			}
			VxHbvhHierarchy::Flat => {
				self.hbvh.flat.remove(id);
			}
		}
		
		if let Some(parent_id) = widget.parent() {
			if let Some(parent) = self.widgets.get_mut(parent_id) {
				parent.remove_child(id);
			}
		}

		Some(widget)
	}

	pub(crate) fn check_dirty(&mut self, res: &mut VxAppResource) -> VxDirtyCheckResult {
		let mut result = VxDirtyCheckResult::None;
		let has_immediate_widget = self.has_immediate_widget();
		let mut queue = self.dirty_queue.take();
		let mut bounding_rect_creator = VxBoundingRectCreator::new(res);
		let mut process_context = VxDirtyProcessContext::new(
			&mut self.widgets,
			&mut bounding_rect_creator,
			self.window_size,
			&mut self.box_layout_resolver,
			&mut self.hbvh
		);
		for (id, flag) in queue.drain() {
			result.merge(process_context.process_update(id, flag));
		}
		if result == VxDirtyCheckResult::None && has_immediate_widget {
			result = VxDirtyCheckResult::OnlyImmediate;
		}

		process_context.finish();

		result
	}

	fn traverse_widget_at(&self, target_bvh: &VxSpatialIndex<VxWidgetId>, at: VxVec2) -> Option<VxWidgetId> {
		let result_widgets = target_bvh.hit_test(at);

		let traverse_result = result_widgets.into_iter()
			.filter_map(|id| {
				let widget = self.widgets.get(id)?;
				if widget.is_visible() {
					Some((VxSpatialTraverseResult::new(id, widget.hbvh_hierarchy()), widget.z_value()))
				} else {
					None
				}
			})
			.max_by_key(|(_, z)| *z)
			.map(|(traverse_result, _)| traverse_result)?;

		let (id, hierarchy) = traverse_result.decompose();

		match hierarchy {
			VxHbvhHierarchy::Parent => {
				let hbvh = self.hbvh.hierarchical.get(&id)?;
				let pos = Self::calc_absolute_target_widget_pos(&self.widgets, id);
				return self.traverse_widget_at(hbvh, at.to_local(pos));
			}
			_ => return Some(id)
		}
	}

	fn calc_absolute_target_widget_pos(widgets: &VxGenVector<Box<dyn VxWidget>>, mut target_id: VxWidgetId) -> VxVec2 {
		let mut result = VxVec2::default();
		while let Some(widget) = widgets.get(target_id) {
			result += widget.pos();
			if let Some(parent_id) = widget.parent() {
				target_id = parent_id;
			} else {
				break;
			}
		}
		result
	}
}

impl VxScene {
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
		self.select_container.set_selected_single(current_id);
		self.send_event_to_widget(
			current_id,
			event,
			|w, e | w.mouse_press_event(e)
		)
	}
	pub fn mouse_release_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let current_id = self.find_widget_at(event.pos());
		self.select_container.set_selected_single(current_id);
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.mouse_release_event(e)
		)
	}
	pub fn mouse_move_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let new_hover_id = self.find_widget_at(event.pos());
		let old_hover_id = self.select_container.hovered();

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
			self.select_container.set_hovered(new_hover_id);
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
	pub fn key_press_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		let current_id = self.select_container.selected_single();
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.key_press_event(e)
		)
	}
	pub fn key_release_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		let current_id = self.select_container.selected_single();
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.key_release_event(e)
		)
	}

	pub fn resized_event(&mut self, window_size: VxSize) -> VxEventResult {
		self.window_size = window_size;
		self.top_level_widgets.iter().for_each(|id| {
			if let Some(widget) = self.widgets.get(*id) {
				widget.context().set_dirty(VxDirtyFlag::REBUILD_ALL);
			}
		});
		VxEventResult::Accept
	}
}


pub(crate) struct VxDirtyProcessContext<'a> {
	widgets: &'a mut VxGenVector<Box<dyn VxWidget>>,
	bounding_rect_creator: &'a mut VxBoundingRectCreator<'a>,
	window_size: VxSize,
	box_layout_resolver: &'a mut VxBoxLayoutResolver,
	hbvh: &'a mut VxHbvh<VxWidgetId>,
}

impl<'a> VxDirtyProcessContext<'a> {
	pub(crate) fn new(
		widgets: &'a mut VxGenVector<Box<dyn VxWidget>>,
		bounding_rect_creator: &'a mut VxBoundingRectCreator<'a>,
		window_size: VxSize,
		box_layout_resolver: &'a mut VxBoxLayoutResolver,
		hbvh: &'a mut VxHbvh<VxWidgetId>
	) -> Self {
		Self { widgets, bounding_rect_creator, window_size, box_layout_resolver, hbvh }
	}

	pub(crate) fn process_update(&mut self, target_id: VxWidgetId, flag: VxDirtyFlag) -> VxDirtyCheckResult {
		let mut result = VxDirtyCheckResult::None;
		if flag == VxDirtyFlag::CLEAN { return VxDirtyCheckResult::None; }
		if flag.contains(VxDirtyFlag::LAYOUT) {
			self.process_update_layout(target_id);
			result = VxDirtyCheckResult::All;
		}
		if flag.contains(VxDirtyFlag::REPAINT) {
			result = VxDirtyCheckResult::All;
		}
		let update_mode = self.widgets.get(target_id)
			.map(|w| w.update_mode())
			.unwrap_or(VxRenderMode::Retained);

		if update_mode == VxRenderMode::Retained {
			if let Some(w) = self.widgets.get(target_id) {
				for child_id in w.children() {
					if let Some(child) = self.widgets.get(*child_id) {
						child.set_dirty(flag);
					}
				}
			}
		}

		if result == VxDirtyCheckResult::None && update_mode == VxRenderMode::Immediate {
			result = VxDirtyCheckResult::OnlyImmediate
		}

		result
	}

	pub(crate) fn finish(&mut self) {
		let need_update_layouts = self.box_layout_resolver.take_need_update_layout_id();

		for id in need_update_layouts {
			let rect = {
				if let Some(widget) = self.widgets.get(id) {
					if widget.parent().is_none() {
						VxRect::from_pos_size(VxVec2::default(), self.window_size)
					} else {
						widget.bounding_rect().with_pos(VxVec2::default())
					}
				} else {
					VxRect::default()
				}
			};

			self.box_layout_resolver.resolve_container(
				self.bounding_rect_creator,
				id,
				rect,
				self.widgets
			);
		}

		for (id, rect) in self.box_layout_resolver.take_computed_rects() {
			if let Some(widget) = self.widgets.get_mut(id) {
				widget.set_block_dirty(true);
				widget.set_pos(rect.pos());
				widget.context_mut().set_bounding_rect(rect.with_pos(VxVec2::default()));
				widget.set_block_dirty(false);

				self.process_update_hbvh(id, rect);
			}
		}

		for flag in self.hbvh.need_update_index.drain() {
			match flag {
				VxSpatialUpdateFlag::Flat => self.hbvh.flat.optimize(),
				VxSpatialUpdateFlag::Hierarchical(id) => {
					if let Some(bvh) = self.hbvh.hierarchical.get_mut(&id) {
						bvh.optimize();
					}
				}
			}
		}
	}
}

impl<'a> VxDirtyProcessContext<'a> {
	fn find_outermost_layout_widget(&self, mut id: VxWidgetId) -> Option<VxWidgetId> {
		let mut result = None;

		while let Some(widget) = self.widgets.get(id) {
			if widget.as_box_layout().is_some() {
				result = Some(id);
			} else if result.is_some() {
				return result;
			}

			if let Some(parent_id) = widget.parent() {
				id = parent_id;
			} else {
				break;
			}
		}

		result
	}

	fn process_update_layout(&mut self, target_id: VxWidgetId) {
		if let Some(result) = self.find_outermost_layout_widget(target_id) {
			self.box_layout_resolver.add_need_update_layout_id(result);
		}
	}

	fn process_update_hbvh(&mut self, target_id: VxWidgetId, rect: VxRect) {
		let Some(widget) = self.widgets.get(target_id) else { return; };

		match widget.hbvh_hierarchy() {
			VxHbvhHierarchy::Child => {
				if let Some(hbvh_parent) = widget.context().hbvh_hierarchy_parent() {
					if let Some(bvh) = self.hbvh.hierarchical.get_mut(&hbvh_parent) {
						bvh.update_at(target_id, rect);
						self.hbvh.need_update_index.insert(VxSpatialUpdateFlag::Hierarchical(hbvh_parent));
					}
				}
			}
			_ => {
				self.hbvh.flat.update_at(target_id, rect.with_pos(VxScene::calc_absolute_target_widget_pos(self.widgets, target_id)));
				self.hbvh.need_update_index.insert(VxSpatialUpdateFlag::Flat);
			}
		}
	}
}