use std::collections::HashMap;

use swash::{scale::ScaleContext};

use crate::{
	core::msdf::{
		VxFontDataGenerator,
		VxMsdfGenerator, VxTextBoundingSizeResult
	},
	types::{
		color::VxColorU8,
		genelational_vector::{VxGenIndex, VxGenVector},
		geometry::{
			VxRect,
			VxSize,
			VxVec2
		},
		texture::VxImage
	}
};

pub(crate) const FALLBACK_FONT: &[u8] = include_bytes!("../../../qd-varallax/src/assets/NotoSans-Regular.ttf");

// maxrect packer
pub struct VxMaxRectsPacker {
	size: VxSize,
	pub free_rects: Vec<VxRect>,
	padding: f32,
}

impl VxMaxRectsPacker {
	pub fn new(size: VxSize, padding: f32) -> Self {
		Self {
			size,
			free_rects: vec![VxRect::from_pos_size(VxVec2::from_i32(0, 0), size)],
			padding
		}
	}

	pub fn insert(&mut self, size: VxSize) -> Option<VxRect> {
		let need_w = size.width() + self.padding;
		let need_h = size.height() + self.padding;

		let mut best_index = None;
		let mut min_short_side_fit = f32::MAX;
		let mut min_long_side_fit = f32::MAX;

		for (i, free) in self.free_rects.iter().enumerate() {
			if free.width() >= need_w && free.height() >= need_h {
				let leftover_w = (free.width() - need_w).abs();
				let leftover_h = (free.height() - need_h).abs();
				let short_side_fit = leftover_w.min(leftover_h);
				let long_side_fit = leftover_w.max(leftover_h);

				if short_side_fit < min_short_side_fit || (short_side_fit == min_short_side_fit && long_side_fit < min_long_side_fit) {
					min_short_side_fit = short_side_fit;
					min_long_side_fit = long_side_fit;
					best_index = Some(i);
				}
			}
		}

		let best_free = best_index.map(|i| self.free_rects[i])?;
		let used_rect = VxRect::new(best_free.x(), best_free.y(), need_w, need_h);

		let mut n = self.free_rects.len();
		let mut i = 0;
		while i < n {
			if self.split_free_rect(i, used_rect) {
				self.free_rects.swap_remove(i);
				n -= 1;
			} else {
				i += 1;
			}
		}

		self.remove_free_rects();

		//パディングなしのRect
		Some(VxRect::new(used_rect.x(), used_rect.y(), size.width(), size.height()))
	}

	fn split_free_rect(&mut self, index: usize, used: VxRect) -> bool {
		let free = self.free_rects[index];

		if !free.intersects(used) {
			return false;
		}

		// left
		if used.left() > free.left() {
			let mut r = free;
			r.set_width(used.left() - free.left());
			self.free_rects.push(r);
		}
		// right
		if used.right() < free.right() {
			let mut r = free;
			r.set_x(used.right());
			r.set_width(free.right() - used.right());
			self.free_rects.push(r);
		}
		// top
		if used.top() > free.top() {
			let mut r = free;
			r.set_height(used.top() - free.top());
			self.free_rects.push(r);
		}
		// bottom
		if used.bottom() < free.bottom() {
			let mut r = free;
			r.set_y(used.bottom());
			r.set_height(free.bottom() - used.bottom());
			self.free_rects.push(r);
		}

		true
	}

	fn remove_free_rects(&mut self) {
		let mut i = 0;
		while i < self.free_rects.len() {
			let mut j = i + 1;
			while j < self.free_rects.len() {
				if self.free_rects[i].encloses(self.free_rects[j]) {
					self.free_rects.remove(j);
				} else if self.free_rects[j].encloses(self.free_rects[i]) {
					self.free_rects.remove(i);
					i -= 1;
					break;
				} else {
					j += 1;
				}
			}
			i += 1;
		}
	}
}


#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct VxGlyphInfo {
	/// アトラス上の位置(ピクセル)
	pub atlas_rect: VxRect,
	/// アトラス上のUV位置
	pub uv_rect: VxRect,
	/// オフセット
	pub bearing_x: f32,
	/// オフセット
	pub bearing_y: f32,
	/// 次の文字への距離(移動量)
	pub advance: f32,
	/// アトラスのパディング
	pub padding: f32,
}

pub struct VxAtlasRectResult {
	pub atlas_id: VxGenIndex,
	pub rect: VxRect,
	pub msdf: VxImage,
	pub ch: char,
}

pub struct VxAtlasAddResult {
	pub added_rect: Vec<VxAtlasRectResult>,
	pub failed_char: Option<Vec<VxTextBoundingSizeResult>>
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct VxVerticalMetrics {
	pub line_height: f32,
	pub ascent: f32,
	pub descent: f32,
}

impl VxVerticalMetrics {
	pub fn new(line_height: f32, ascent: f32, descent: f32) -> Self {
		Self { line_height, ascent, descent }
	}
	pub fn is_empty(&self) -> bool {
		self.line_height <= 0.0 && self.ascent <= 0.0 && self.descent <= 0.0
	}
	pub fn create_line_height(&self) -> f32 {
		self.line_height + self.ascent - self.descent
	}
}

pub struct VxFontAtlas {
	pub font_family: VxFontFamilyName,
	pub glyphs: HashMap<char, VxGlyphInfo>,
	pub texture_data: VxImage,
	pub packer: VxMaxRectsPacker,
	pub is_registered: bool,
	pub is_full: bool,
	pub id: VxGenIndex,
}

impl VxFontAtlas {
	pub fn new_empty(family: impl Into<VxFontFamilyName>, atlas_size: u32) -> Self {
		let size = VxSize::from_u32(atlas_size, atlas_size);
		Self {
			font_family: family.into(),
			glyphs: HashMap::new(),
			texture_data: VxImage::new(size, VxColorU8::rgb(0, 0, 0)),
			packer: VxMaxRectsPacker::new(size, 4.0),
			is_registered: false,
			is_full: false,
			id: VxGenIndex { index: 0, generation: 0 }
		}
	}

	pub fn add_missing_chars(
		&mut self,
		context: &mut ScaleContext,
		font_data: &[u8],
		chars: &str,
		msdf_size: f32
	) -> Option<VxAtlasAddResult> {
		if !self.is_registered { return None; }
		let missing_chars = self.create_missing_chars(chars);
		if missing_chars.is_empty() { return None; };

		// 文字の実際のサイズを取得
		let mut text_bounding_size = VxFontDataGenerator::create_text_bounding_size(
			context, font_data, msdf_size, VxMsdfGenerator::RANGE, &missing_chars
		);

		// 高さを優先してソート
		text_bounding_size.sort_by(|a, b| {
			b.size.height().total_cmp(&a.size.height())
				.then_with(|| b.size.width().total_cmp(&a.size.width()))
		});

		// MSDFを生成
		let mut packing_rect = vec![];
		let mut failed_chars = vec![];
		for mut size_result in text_bounding_size.drain(..) {
			if let Some(outline) = size_result.outline.take() {
				if let Some(rect) = self.packer.insert(size_result.size) {
					let bounding_rect = VxFontDataGenerator::create_bounding_rect(&outline);
					let bearing_x = VxFontDataGenerator::create_bearing_x(bounding_rect, VxMsdfGenerator::RANGE);
					let bearing_y = VxFontDataGenerator::create_bearing_y(bounding_rect, VxMsdfGenerator::RANGE);

					let msdf_texture = VxMsdfGenerator::create_msdf_from_outline(outline);
					let atlas_size = self.texture_data.size().width();
					let (img_w, img_h) = msdf_texture.size().to_tuple();

					let uv_rect = VxRect::new(
						rect.x() / atlas_size,
						rect.y() / atlas_size,
						img_w / atlas_size,
						img_h / atlas_size
					);				

					self.glyphs.insert(size_result.ch, VxGlyphInfo {
						atlas_rect: rect,
						uv_rect,
						bearing_x,
						bearing_y,
						advance: size_result.advance,
						padding: VxMsdfGenerator::RANGE as f32,
					});

					packing_rect.push(VxAtlasRectResult {
						atlas_id: self.id,
						rect,
						msdf: msdf_texture,
						ch: size_result.ch,
					});
				} else {
					size_result.outline = Some(outline);
					self.is_full = true;
					failed_chars.push(size_result);
				}
			} else {
				self.glyphs.insert(size_result.ch, VxGlyphInfo {
					atlas_rect: VxRect::new(0.0, 0.0, 0.0, msdf_size),
					uv_rect: VxRect::default(),
					bearing_x: 0.0,
					bearing_y: 0.0,
					advance: size_result.advance,
					padding: 0.0,
				});
			}
		}
		if failed_chars.is_empty() {
			Some(VxAtlasAddResult { added_rect: packing_rect, failed_char: None })
		} else {
			Some(VxAtlasAddResult { added_rect: packing_rect, failed_char: Some(failed_chars) })
		}
	}

	fn create_missing_chars(&self, chars: &str) -> Vec<char> {
		let mut missing_chars = vec![];
		for ch in chars.chars() {
			if !self.glyphs.contains_key(&ch) {
				missing_chars.push(ch);
			}
		}
		missing_chars
	}

	pub fn is_full(&self) -> bool { self.is_full }
	pub fn is_registered(&self) -> bool { self.is_registered }
}

pub type VxFontFamilyName = String;

pub struct VxFontFamily {
	pub name: VxFontFamilyName,
	pub vertical_metrics: VxVerticalMetrics,
	pub atlases: VxGenVector<VxFontAtlas>,
	pub atlas_table: HashMap<char, VxGenIndex>,
}

impl VxFontFamily {
	pub fn new(name: impl Into<VxFontFamilyName>, vertical_metrics: VxVerticalMetrics) -> Self {
		Self { name: name.into(), vertical_metrics, atlases: VxGenVector::new(), atlas_table: HashMap::new() }
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct VxFont {
	family_name: VxFontFamilyName,
	pixel_size: f32,
}

impl VxFont {
	pub fn new(family: impl Into<VxFontFamilyName>, size: f32) -> Self {
		Self {
			family_name: family.into(),
			pixel_size: size,
		}
	}

	pub fn family(&self) -> &VxFontFamilyName {
		&self.family_name
	}
	pub fn pixel_size(&self) -> f32 {
		self.pixel_size
	}
}