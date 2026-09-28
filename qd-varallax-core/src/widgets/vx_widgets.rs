
// use qd_varallax_macro::VxWidgetImpl;

// use crate::{
// 	abstracts::{abstract_layouts::VxSizeHint, abstract_widgets::*}, types::{
// 		color::VxColor, event::VxEventResult, geometry::VxRect, texture::VxTexture
// 	},
// };


// #[derive(VxWidgetImpl)]
// pub struct VxRectWidget {
// 	#[attr(stat)]
// 	stat: VxWidgetStats,
// 	rect: VxRect,
// 	color: VxColor,
// }

// impl VxWidget for VxRectWidget {
// 	fn size_hint(&mut self, _: &mut crate::abstracts::abstract_layouts::VxBoundingRectCreator) -> VxSizeHint {
// 		VxSizeHint::Content(self.rect.size())
// 	}

// 	fn paint(&mut self, painter: &mut crate::painter::painter::VxPainter) {
// 		painter.draw_rect(self.bounding_rect(), self.color);
// 	}

// 	fn mouse_press_event(&mut self, event: &crate::types::event::VxMouseEvent) -> VxEventResult {
// 		println!("{:?}", event.pos());
// 		VxEventResult::Accept
// 	}
// }

// impl VxRectWidget {
// 	pub fn new(rect: VxRect, color: VxColor, parent: Option<VxWidgetId>) -> Self {
// 		Self {
// 			stat: VxWidgetStats::new(parent),
// 			rect,
// 			color,
// 		}
// 	}
// }

// #[derive(VxWidgetImpl)]
// pub struct VxTextureWidget {
// 	#[attr(stat)]
// 	stat: VxWidgetStats,
// 	rect: VxRect,
// 	texture: VxTexture,
// 	opacity: f32,
// }

// impl VxWidget for VxTextureWidget {
// 	fn size_hint(&mut self, _: &mut crate::abstracts::abstract_layouts::VxBoundingRectCreator) -> VxSizeHint {
// 		VxSizeHint::Content(self.rect.size())
// 	}
// 	fn paint(&mut self, painter: &mut crate::painter::painter::VxPainter) {
// 		painter.draw_texture(
// 			self.rect,
// 			VxColor::from_hex(0xFFFFFF).with_alpha(self.opacity),
// 			&self.texture()
// 		);
// 	}

// 	fn register_texture_event(&mut self, gpu: &crate::core::gpu_resource::VxGpuResource, system: &mut crate::core::systems::VxTextureSystem) {
// 		system.register_texture(gpu, &mut self.texture);
// 	}
// }

// impl VxTextureWidget {
// 	pub fn new(rect: VxRect, texture: VxTexture, parent:Option<VxWidgetId>) -> Self {
// 		Self {
// 			stat: VxWidgetStats::new(parent),
// 			rect,
// 			texture,
// 			opacity: 1.0
// 		}
// 	}
// 	pub fn texture(&self) -> &VxTexture {
// 		&self.texture
// 	}

// 	pub fn set_texture(&mut self, tex: VxTexture) {
// 		self.texture = tex;
// 	}
// 	pub fn set_rect(&mut self, rect: VxRect) {
// 		self.rect = rect;
// 	}
// 	pub fn set_opacity(&mut self, opacity: f32) {
// 		self.opacity = opacity.clamp(0.0, 1.0);
// 	}
// }