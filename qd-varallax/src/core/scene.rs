
use crate::{
	abstractions::{abstract_layouts::VxAlignment, abstract_widgets::{
		VxWidget, VxWidgetHandler, VxWidgetId
	}},
	core::{
		bvh::VxSpatialIndex,
		resource::VxAppResources,
		vx_event::{
			VxEventResult,
			VxKeyEvent,
			VxMouseEvent
		}
	},
	painter::painter::VxPainter,
	types::{
		genelational_vector::VxGenVector,
		geometry::{VxRect, VxSize, VxVec2}, transform::VxTransform
	},
	utils::convert::VxUtilConverter
};


pub struct VxScene {
	widgets: VxGenVector<Box<dyn VxWidget>>,
	top_level_widgets: Vec<VxWidgetId>,

	spatial_index: VxSpatialIndex,

	current_selected_widget: Option<VxWidgetId>,
	current_hovered_widget: Option<VxWidgetId>,
}

impl VxScene {
	pub fn new() -> Self {
		Self {
			widgets: VxGenVector::new(),
			top_level_widgets: vec![],
			spatial_index: VxSpatialIndex::new(),
			current_selected_widget: None,
			current_hovered_widget: None,
		}
	}

	// private methods
	fn paint_widget(&mut self, res: &mut VxAppResources, id: VxWidgetId, painter: &mut VxPainter) {
		let Some(widget) = self.widgets.get_mut(id.id()) else { return; };
		if !widget.is_visible() {
			return;
		}

		widget.register_texture_event(&res.gpu, &mut res.textures);

		widget.paint(painter);
		painter.set_vertex_z_value(widget.z_value());
		let pos = widget.pos();
		let z = widget.z_value();
		let bounding_rect = widget.bounding_rect();

		let children = widget.children();
		
		for child in children.clone() {
			let Some(c) = self.widgets.get_mut(child.id()) else { continue; };
			c.set_z_value(z + 1);
			painter.push_tranform(
				VxTransform::new(
					Self::apply_alignment_to_rect(pos, c.bounding_rect(), c.stats().alignment(), bounding_rect),
					VxSize::default(),
					VxSize::new(1.0, 1.0),
					0.0.into(),
					VxVec2::default()
			));
			self.paint_widget(res, child, painter);
			painter.pop_transform();
		}
	}

	fn apply_alignment_to_rect(pos: VxVec2, rect: VxRect, alignment: VxAlignment, bounding_rect: VxRect) -> VxVec2 {
		match alignment {
			VxAlignment::LeftTop => pos,
			VxAlignment::LeftCenter => {
				bounding_rect.left_center() - rect.left_center()
			}
			VxAlignment::LeftBottom => {
				bounding_rect.left_bottom() - rect.left_bottom()
			}
			VxAlignment::TopCenter => {
				bounding_rect.top_center() - rect.top_center()
			}
			VxAlignment::Center => {
				bounding_rect.center() - rect.center()
			}
			VxAlignment::BottomCenter => {
				bounding_rect.bottom_center() - rect.bottom_center()
			}
			VxAlignment::RightTop => {
				bounding_rect.right_top() - rect.right_top()
			}
			VxAlignment::RightCenter => {
				bounding_rect.right_center() - rect.right_center()
			}
			VxAlignment::RightBottom => {
				bounding_rect.right_bottom() - rect.right_bottom()
			}
			VxAlignment::CustomAlignment { pos } => {
				bounding_rect.left_top() - pos
			}
		}
	}

	// 指定場所にウィジェットがあるか特定
	fn find_widget_at(&mut self, pos: VxVec2) -> Option<VxWidgetId> {
		let result = self.spatial_index.hit_test(pos);
		if result.is_empty() { return None; }

		result.into_iter()
			.filter_map(|id| {
				let widget = self.widgets.get(id.id())?;
				if widget.is_visible() && widget.bounding_rect().contains(pos) {
					Some((widget.z_value(), id))
				} else {
					None
				}
			})
			.max_by_key(|(z, _)| *z)
			.map(|(_, id)| id)
	}

	pub(crate) fn check_dirty(&mut self) -> bool {
		let mut dirty = false;
		for (id, widget) in self.widgets.iter_with_id_mut() {
			if widget.is_dirty() {
				dirty = true;
				widget.set_dirty(false);
				self.spatial_index.udpate_at(
					VxWidgetId::new(id),
					VxUtilConverter::rect_to_aabb(&widget.bounding_rect())
				);
			}
		}

		if dirty {
			self.spatial_index.finalize_update();
		}

		dirty
	}

	// methods
	pub fn add_widget(&mut self, mut widget: Box<dyn VxWidget>) -> VxWidgetId {
		let children = widget.stats_mut().children_widgets();

		let id = VxWidgetId::new(self.widgets.insert(widget));
		let widget_ref = self.widgets.get_mut(id.id()).unwrap();
		widget_ref.stats_mut().set_widget_id(id);

		if widget_ref.parent().is_none() {
			self.top_level_widgets.push(id);
		}

		for mut w in children {
			w.set_parent(id);
			let child_id = self.add_widget(w);

			if let Some(parent) = self.widgets.get_mut(id.id()) {
				parent.stats_mut().add_child(child_id);
			}
		}

		id
	}

	pub fn get_widget<W: VxWidget>(&mut self, widget: &VxWidgetHandler<W>) -> Option<&mut W> {
		self.widgets.get_mut(widget.id().id())?
			.as_any_mut()
			.downcast_mut::<W>()
	}

	pub(crate) fn refresh_spatial_index(&mut self) {
		let mut data = vec![];
		for (index, widget) in self.widgets.iter_with_id() {
			let rect = widget.bounding_rect();
			let aabb = VxUtilConverter::rect_to_aabb(&rect);
			data.push((VxWidgetId::new(index), aabb));
		}
		self.spatial_index.rebuild_bvh(&data);
	}

	// イベントハンドラー
	fn send_event_to_widget<E>(
		&mut self,
		start_id: Option<VxWidgetId>,
		event: &E,
		handler: impl Fn(&mut Box<dyn VxWidget>, &E) -> VxEventResult
	) -> VxEventResult {
		let mut current_id = start_id;

		while let Some(id) = current_id {
			let widget = self.widgets.get_mut(id.id()).unwrap();
			let result = handler(widget, event);
			if result == VxEventResult::Accept {
				return VxEventResult::Accept;
			}
			current_id = *self.widgets.get(id.id()).unwrap().parent();
		}
		VxEventResult::Ignore
	}
	// Events
	pub fn paint_event(&mut self, res: &mut VxAppResources, painter: &mut VxPainter) {
		for id in self.top_level_widgets.clone() {
			self.paint_widget(res, id, painter);
		}
	}
	// MouseEvents
	pub fn mouse_press_event(&mut self, event: &VxMouseEvent)  -> VxEventResult {
		let current_id = self.find_widget_at(event.pos());
		self.current_selected_widget = current_id;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e | w.mouse_press_event(e)
		)
	}
	pub fn mouse_release_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let current_id = self.find_widget_at(event.pos());
		self.current_selected_widget = current_id;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.mouse_release_event(e)
		)
	}
	pub fn mouse_move_event(&mut self, event: &VxMouseEvent) -> VxEventResult {
		let new_hover_id = self.find_widget_at(event.pos());
		let old_hover_id = self.current_hovered_widget;

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
			self.current_hovered_widget = new_hover_id;
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
		let current_id = self.current_selected_widget;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.key_press_event(e)
		)
	}
	pub fn key_release_event(&mut self, event: &VxKeyEvent) -> VxEventResult {
		let current_id = self.current_selected_widget;
		self.send_event_to_widget(
			current_id,
			event,
			|w, e| w.key_release_event(e)
		)
	}
}