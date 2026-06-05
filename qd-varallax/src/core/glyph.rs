use core::f32;
use std::hash::{
	Hash,
	Hasher
};

use crate::types::{
	genelational_vector::VxGenIndex,
	geometry::{
		VxRect,
		VxSize,
		VxVec2
	},
};

pub(crate) const FALLBACK_FONT: &[u8] = include_bytes!("../../../qd-varallax/src/assets/NotoSans-Regular.ttf");
pub(crate) const FALLBACK_FONT_NAME: &str = "NotoSans";

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
			padding,
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

				if short_side_fit < min_short_side_fit ||
					(short_side_fit == min_short_side_fit && long_side_fit < min_long_side_fit)
				{
					min_short_side_fit = short_side_fit;
					min_long_side_fit = long_side_fit;
					best_index = Some(i);
				}
			}
		}

		let best_free = best_index.map(|i| self.free_rects[i])?;

		let used_rect = VxRect::new(best_free.x(), best_free.y(), need_w, need_h);
		let mut old_free_rects = std::mem::take(&mut self.free_rects);
		for free in old_free_rects.drain(..) {
			self.split_free_rect(free, used_rect);
		}

		self.remove_free_rects();

		Some(VxRect::from_pos_size(used_rect.pos(), size))
	}

	fn split_free_rect(&mut self, free: VxRect, used: VxRect) {
		if !free.intersects(used) {
			self.free_rects.push(free);
			return;
		}

		// left
		if used.left() > free.left() {
			let mut r = free;
			r.set_width(used.left() - free.left());
			if !r.size().is_empty() { self.free_rects.push(r) };
		}
		// right
		if used.right() < free.right() {
			let mut r = free;
			r.set_x(used.right());
			r.set_width(free.right() - used.right());
			if !r.size().is_empty() { self.free_rects.push(r) };
		}
		// top
		if used.top() > free.top() {
			let mut r = free;
			r.set_height(used.top() - free.top());
			if !r.size().is_empty() { self.free_rects.push(r) };
		}
		// bottom
		if used.bottom() < free.bottom() {
			let mut r = free;
			r.set_y(used.bottom());
			r.set_height(free.bottom() - used.bottom());
			if !r.size().is_empty() { self.free_rects.push(r) };
		}
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

					i = i.saturating_sub(1);
					break;
				} else {
					j += 1;
				}
			}
			i += 1;
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

impl VxGlyphInfo {
	pub fn new(
		atlas_rect: VxRect,
		uv_rect: VxRect,
		bearing_x: f32,
		bearing_y: f32,
		advance: f32,
		padding: f32,
	) -> Self {
		Self {
			atlas_rect,
			uv_rect,
			bearing_x,
			bearing_y,
			advance,
			padding,
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct VxVerticalMetrics {
	pub line_height: f32,
	pub ascent: f32,
	pub descent: f32,
}

impl VxVerticalMetrics {
	pub fn new(line_height: f32, ascent: f32, descent: f32) -> Self {
		Self {
			line_height,
			ascent,
			descent,
		}
	}
	#[inline]
	pub fn is_empty(&self) -> bool {
		self.line_height <= 0.0 && self.ascent <= 0.0 && self.descent <= 0.0
	}
	#[inline]
	pub fn create_line_height(&self) -> f32 {
		self.ascent + self.descent + self.line_height
	}
}

pub type VxFontFamilyHash = u64;

pub struct VxFontAtlas {
	pub id: VxGenIndex,
	pub packer: VxMaxRectsPacker,
	pub is_full: bool,
}

impl VxFontAtlas {
	pub fn new_empty(size: u32) -> Self {
		Self {
			id: VxGenIndex::default(),
			packer: VxMaxRectsPacker::new(VxSize::from_u32(size, size), 4.0),
			is_full: false,
		}
	}

	#[inline]
	pub fn is_full(&self) -> bool {
		self.is_full
	}
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VxFont {
	family_hash: VxFontFamilyHash,
	pixel_size: f32,
}

impl VxFont {
	#[inline]
	pub fn new(family: VxFontFamilyHash, size: f32) -> Self {
		Self {
			family_hash: family,
			pixel_size: size,
		}
	}

	#[inline]
	pub fn from_family_str(family: &str, size: f32) -> Self {
		Self {
			family_hash: Self::hash(family),
			pixel_size: size,
		}
	}

	#[inline]
	pub fn family(&self) -> VxFontFamilyHash {
		self.family_hash
	}
	#[inline]
	pub fn pixel_size(&self) -> f32 {
		self.pixel_size
	}

	pub fn hash(text: &str) -> u64 {
		let mut hasher = ahash::AHasher::default();
		text.hash(&mut hasher);
		hasher.finish()
	}
}
