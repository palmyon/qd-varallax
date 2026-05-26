use std::path::Path;

use image::{DynamicImage, ImageReader};

use crate::types::{
	color::{VxColorChannel, VxColorChannelSwap, VxColorU8}, genelational_vector::VxGenIndex, geometry::{
		VxAspectMode,
		VxSize,
		VxVec2
	}
};

pub struct VxImage {
	pixels: Vec<VxColorU8>,
	size: VxSize,
	path: String,
}

impl Default for VxImage {
	fn default() -> Self {
		Self::new(VxSize::default(), VxColorU8::from_hex(0xFF00FF))
	}
}

impl VxImage {
	pub fn new(size: VxSize, color: VxColorU8) -> Self {
		let len = (size.width() * size.height()) as usize;
		let pixels = vec![color; len];
		Self { pixels, size, path: String::new() }
	}

	/// QD-Varallax VxImage > method > from_file() \
	/// 画像をパスから読み込む。 \
	/// `size`が指定されている時、`mode`に合わせたスケーリングを行う。 \
	/// `size`を指定する場合、`mode`も指定する必要がある。
	/// # Args
	/// `size: Option<VxSize>` 読み込みたいサイズ \
	/// `mode: Option<VxAspectMode>` アスペクト比モード
	pub fn from_file(path: &Path, size: Option<VxSize>, mode: Option<VxAspectMode>) -> Result<Self, String> {
		let reader = ImageReader::open(path)
			.map_err(|e| e.to_string())?
			.with_guessed_format()
			.map_err(|e| e.to_string())?;

		let img = reader.decode().map_err(|e| e.to_string())?;

		let scaled_img = {
			if let (Some(s), Some(m)) = (size, mode) {
				Self::scale_img(img, s, m)
			} else {
				img
			}
		};

		let rgba = scaled_img.to_rgba8();
		let (w, h) = rgba.dimensions();
		let pixels = rgba.pixels()
			.map(|p| VxColorU8::rgba(p[0], p[1], p[2], p[3]))
			.collect();

		let str_path = {
			if let Some(p) = path.to_str() {
				p.to_string()
			} else {
				"".to_string()
			}
		};

		Ok(Self{
			pixels,
			size: VxSize::from_u32(w, h),
			path: str_path,
		})
	}

	pub fn from_raw_rgb(data: Vec<u8>, size: VxSize) -> Self {
		let mut pixels = Vec::with_capacity(data.len() / 3);
		for rgb in data.chunks(3) {
			pixels.push(VxColorU8::rgba(rgb[0], rgb[1], rgb[2], 255));
		}
		Self {
			pixels,
			size,
			path: "".into(),
		}
	}

	// getter
	pub fn size(&self) -> VxSize {
		self.size
	}
	pub fn path(&self) -> &String {
		&self.path
	}
	pub fn is_empty(&self) -> bool {
		self.pixels.is_empty()
	}
	pub fn pixels(&self) -> &Vec<VxColorU8> {
		&self.pixels
	}
	pub fn pixels_mut(&mut self) -> &mut Vec<VxColorU8> {
		&mut self.pixels
	}

	pub fn to_raw_rgba8(&mut self) -> Vec<u8> {
		let colors = std::mem::take(&mut self.pixels);
		let mut raw = Vec::with_capacity(colors.len() * 4);
		for color in colors {
			let (r, g, b, a) = color.to_tuple();
			raw.push(r);
			raw.push(g);
			raw.push(b);
			raw.push(a);
		}
		raw
	}

	pub fn to_raw_rgb(&self) -> Vec<u8> {
		let mut raw = Vec::with_capacity(self.pixels.len() * 3);
		for color in &self.pixels {
			let (r, g, b, _) = color.to_tuple();
			raw.push(r);
			raw.push(g);
			raw.push(b);
		}
		raw
	}

	pub fn as_raw_rgba8(&self) -> &[u8] {
		bytemuck::cast_slice(&self.pixels)
	}

	pub fn pixel(&self, location: VxVec2) -> Option<VxColorU8> {
		let (x, y) = location.to_tuple();
		if x >= self.size().width() || y >= self.size().height() {
			return None;
		}
		Some(self.pixels[(y * self.size.width() + x) as usize])
	}

	pub fn clear_pixels(&mut self) {
		self.pixels.clear();
	}

	pub fn clear_pixels_with_reset(&mut self) {
		self.pixels.clear();
		self.size.set_size(0.0, 0.0);
	}

	pub fn set_color_map(&mut self, map: VxTexColorChannelMap) {
		match map {
			VxTexColorChannelMap::Channel { channel_mode } => {
				for pix in &mut self.pixels {
					pix.set_color_channel(channel_mode);
				}
			}
			VxTexColorChannelMap::Swap { swap_mode } => {
				for pix in &mut self.pixels {
					pix.swap_channel(swap_mode);
				}
			}
		}
	}

	pub fn save(&self, path: impl Into<String>) {
		let pixels = self.as_raw_rgba8();
		let _ = image::save_buffer(
			path.into(),
			&pixels,
			self.size.width_u32(),
			self.size.height_u32(),
			image::ExtendedColorType::Rgba8,
		);
	}

	// private methods
	fn scale_img(img: DynamicImage, target_size: VxSize, mode: VxAspectMode) -> DynamicImage {
		let scaled_img = {
			match mode {
				VxAspectMode::IgnoreAspectRatio => {
					img.thumbnail_exact(
						target_size.width_u32(),
						target_size.height_u32(),
					)
				},
				_ => {
					let img_size = VxSize::from_u32(img.width(), img.height());
					let scaled = img_size.scaled(target_size, mode);
					img.thumbnail_exact(
						scaled.width_u32(),
						scaled.height_u32(),
					)
				}
			}
		};
		scaled_img
	}
}


pub struct VxTexture {
	id: Option<VxGenIndex>,
	img: Option<VxImage>,
	channel_map: Option<Vec<VxTexColorChannelMap>>,
}

impl Default for VxTexture {
	fn default() -> Self {
		Self {
			id: None,
			img: None,
			channel_map: None,
		}
	}
}

impl VxTexture {
	pub fn new() -> Self {
		Self::default()
	}
	pub fn from_file(path: &Path) -> Result<Self, String> {
		let img = VxImage::from_file(path, None, None)?;
		Ok(Self {
			id: None,
			img: Some(img),
			channel_map: None,
		})
	}
	// getter
	#[inline]
	pub fn id(&self) -> Option<VxGenIndex> {
		self.id
	}
	#[inline]
	pub fn texture(&mut self) -> &mut Option<VxImage> {
		&mut self.img
	}
	#[inline]
	pub fn channel_map(&self) -> &Option<Vec<VxTexColorChannelMap>> {
		&self.channel_map
	}

	pub(crate) fn load_pixel(&mut self) -> Option<Vec<u8>> {
		if let Some(img) = &mut self.img {
			if let Some(maps) = &mut self.channel_map {
				for c in std::mem::take(maps) {
					img.set_color_map(c);
				}
			}
			let raw_pixels = img.to_raw_rgba8();
			return Some(raw_pixels);
		}
		None
	}

	#[inline]
	pub fn add_channel_map(&mut self, map: VxTexColorChannelMap) {
		if let Some(ref mut channels) = self.channel_map {
			channels.push(map);
		}
	}

	#[inline]
	pub fn clear_channel_map(&mut self) {
		if let Some(ref mut channels) = self.channel_map {
			channels.clear();
		}
	}
	#[inline]
	pub fn set_id(&mut self, id: VxGenIndex) {
		self.id = Some(id)
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VxTexColorChannelMap {
	Channel { channel_mode: VxColorChannel },
	Swap { swap_mode: VxColorChannelSwap },
}