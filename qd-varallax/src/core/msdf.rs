use fdsm::{
	bezier::{
		Point,
		Segment,
		scanline::FillRule
	},
	shape::{
		Contour,
		Shape
	},
	transform::Transform
};
use image::RgbImage;
use nalgebra::{
	Affine2,
	Matrix3
};
use swash::{
	FontRef, proxy::MetricsProxy, scale::{
		ScaleContext, Scaler, outline::Outline
	}, zeno::{
		self,
		PathData
	}
};

use crate::{core::glyph::VxVerticalMetrics, types::{
	geometry::{VxRect, VxSize},
	texture::VxImage
}};

pub struct VxMsdfGenResult {
	pub ch: char,
	pub msdf_texture: VxImage,
	pub bearing_x: f32,
	pub bearing_y: f32,
	pub advance: f32,
}

impl VxMsdfGenResult {
	pub fn new(ch: char, msdf_texture: VxImage, bearing_x: f32, bearing_y: f32, advance: f32) -> Self {
		Self { ch, msdf_texture, bearing_x, bearing_y, advance }
	}
}

pub struct VxMsdfGenerator;

impl VxMsdfGenerator {
	pub const RANGE: f32 = 10.0;

	pub fn create_msdf(context: &mut ScaleContext, font_data: &[u8], msdf_size: f32, ch: char) -> Option<VxMsdfGenResult> {
		let font_ref = VxFontDataGenerator::create_fontref(font_data)?;
		let mut scaler = VxFontDataGenerator::create_scaler(context, font_ref, msdf_size);
		let glyph_id = VxFontDataGenerator::create_glyph_id(font_ref, ch);

		// outline and bounding rect
		let outline = VxFontDataGenerator::create_outline(glyph_id, &mut scaler)?;
		let bounding_rect = VxFontDataGenerator::create_bounding_rect(&outline);

		// font metrics data
		let advance = VxFontDataGenerator::create_advance(font_ref, glyph_id, msdf_size);
		let bearing_x = VxFontDataGenerator::create_bearing_x(bounding_rect, Self::RANGE);
		let bearing_y = VxFontDataGenerator::create_bearing_y(bounding_rect, Self::RANGE);
		let padding = VxFontDataGenerator::create_padding(Self::RANGE);

		// generate msdf
		let texture_size = VxFontDataGenerator::create_texture_size(bounding_rect, padding);
		let mut shape = VxFontDataGenerator::create_shape(&outline);
		VxFontDataGenerator::apply_transform_to_shape(&mut shape, texture_size, bounding_rect, Self::RANGE);
		let msdf_texture = VxFontDataGenerator::create_msdf_from_shape(
			shape, texture_size, Self::RANGE,
		);
		let msdf_size = VxSize::from_u32(msdf_texture.width(), msdf_texture.height());

		Some(
			VxMsdfGenResult::new(
				ch,
				VxImage::from_raw_rgb(
					msdf_texture.into_raw(),
					msdf_size
				),
				bearing_x,
				bearing_y,
				advance
			)
		)
	}

	pub fn create_msdf_from_outline(outline: Outline) -> VxImage {
		let bounding_rect = VxFontDataGenerator::create_bounding_rect(&outline);
		let padding = VxFontDataGenerator::create_padding(Self::RANGE);
		let texture_size = VxFontDataGenerator::create_texture_size(bounding_rect, padding);
		let mut shape = VxFontDataGenerator::create_shape(&outline);
		VxFontDataGenerator::apply_transform_to_shape(&mut shape, texture_size, bounding_rect, Self::RANGE);
		let msdf_texture = VxFontDataGenerator::create_msdf_from_shape(
			shape, texture_size, Self::RANGE
		);
		let msdf_size = VxSize::from_u32(msdf_texture.width(), msdf_texture.height());
		VxImage::from_raw_rgb(msdf_texture.into_raw(), msdf_size)
	}
}


pub(crate) struct VxFontDataGenerator;

impl VxFontDataGenerator {
	pub fn create_fontref(data: &[u8]) -> Option<FontRef<'_>> {
		FontRef::from_index(data, 0)
	}
	pub fn create_scaler<'a>(context: &'a mut ScaleContext, font_ref: FontRef<'a>, font_size: f32) -> Scaler<'a> {
		context.builder(font_ref).hint(true).size(font_size).build()
	}
	pub fn create_glyph_id(font_ref: FontRef<'_>, ch: char) -> u16 {
		font_ref.charmap().map(ch)
	}
	pub fn create_outline(id: u16, scaler: &mut Scaler<'_>) -> Option<Outline> {
		scaler.scale_outline(id)
	}
	pub fn create_advance(font_ref: FontRef<'_>, id: u16, font_size: f32) -> f32 {
		let bind = [id as i16];
		let metrics = font_ref.glyph_metrics(&bind);
		(metrics.advance_width(id) * font_size) / metrics.units_per_em() as f32
	}
	#[inline]
	pub fn create_bounding_rect(outline: &Outline) -> VxRect {
		let bbox = outline.bounds();
		VxRect::new(bbox.min.x, bbox.min.y, bbox.width(), bbox.height())
	}
	#[inline(always)]
	pub fn create_bearing_x(bounding_rect: VxRect, range: f32) -> f32 {
		bounding_rect.x() - range
	}
	#[inline(always)]
	pub fn create_bearing_y(bounding_rect: VxRect, range: f32) -> f32 {
		bounding_rect.height() - range
	}
	pub fn create_vertical_metrics(font_ref: &FontRef<'_>) -> VxVerticalMetrics {
		let metrics = MetricsProxy::from_font(&font_ref);
		let m = metrics.materialize_metrics(&font_ref, &[]);
		VxVerticalMetrics { line_height: m.leading, ascent: m.ascent, descent: m.descent }
	}
	#[inline]
	pub fn create_padding(range: f32) -> f32 { range * 2.0 }
	#[inline]
	pub fn create_texture_size(bounding_rect: VxRect, padding: f32) -> VxSize {
		let mut size = bounding_rect.size();
		size.set_width(size.width() + padding);
		size.set_height(size.height() + padding);
		size
	}
	pub fn create_shape(outline: &Outline) -> Shape<Contour> {
		let mut contours = vec![];
		let mut segments = vec![];
		let mut start_pos = Point::new(0.0, 0.0);
		let mut last_pos = Point::new(0.0, 0.0);

		for path_element in outline.path().commands() {
			match path_element {
				zeno::Command::MoveTo(p) => {
					if !segments.is_empty() {
						contours.push(Contour { segments: std::mem::take(&mut segments) });
					}
					last_pos = Point::new(p.x as f64, p.y as f64);
					start_pos = last_pos;
				}
				zeno::Command::LineTo(p) => {
					let to = Point::new(p.x as f64, p.y as f64);
					segments.push(Segment::line(last_pos, to));
					last_pos = to;
				}
				zeno::Command::QuadTo(p1, p) => {
					let cp = Point::new(p1.x as f64, p1.y as f64);
					let to = Point::new(p.x as f64, p.y as f64);
					segments.push(Segment::quad(last_pos, cp, to));
					last_pos = to;
				}
				zeno::Command::CurveTo(p1, p2, p) => {
					let cp1 = Point::new(p1.x as f64, p1.y as f64);
					let cp2 = Point::new(p2.x as f64, p2.y as f64);
					let to = Point::new(p.x as f64, p.y as f64);
					segments.push(Segment::cubic(last_pos, cp1, cp2, to));
					last_pos = to;
				}
				zeno::Command::Close => {
					if !segments.is_empty() {
						if last_pos != start_pos {
							segments.push(Segment::line(last_pos, start_pos));
						}
					}
					contours.push(Contour { segments: std::mem::take(&mut segments) });
				}
			}
		}
		if !segments.is_empty() {
			contours.push(Contour { segments });
		}
		Shape { contours }
	}
	pub fn apply_transform_to_shape(shape: &mut Shape<Contour>, texture_size: VxSize, bounding_rect: VxRect, range: f32) {
		let tx = (-bounding_rect.x() + range) as f64;
		let ty = (-bounding_rect.y() + range) as f64;
		shape.transform(&Affine2::from_matrix_unchecked(
			Matrix3::new(
				1.0, 0.0, tx,
				0.0, -1.0, (texture_size.height() as f64) - ty,
				0.0, 0.0, 1.0,
		)));
	}
	pub fn create_msdf_from_shape(shape: Shape<Contour>, texture_size: VxSize, range: f32) -> RgbImage {
		let coloring = Shape::edge_coloring_simple(shape, 0.03, 69441337420);
		let prepared = coloring.prepare();
		let mut texture = RgbImage::new(texture_size.width_u32(), texture_size.height_u32());
		fdsm::generate::generate_msdf(&prepared, range as f64, &mut texture);
		fdsm::render::correct_sign_msdf(&mut texture, &prepared, FillRule::Nonzero);
		texture
	}
	pub fn create_text_bounding_size(
		context: &mut ScaleContext,
		font_data: &[u8],
		font_size: f32,
		range: f32,
		chars: &[char]
	) -> Vec<VxTextBoundingSizeResult> {
		let Some(font_ref) = Self::create_fontref(font_data) else { return vec![]; };
		let mut scaler = Self::create_scaler(context, font_ref, font_size);
		let char_map = font_ref.charmap();
		let padding = Self::create_padding(range);

		chars.iter().map(|&ch| {
			let glyph_id = char_map.map(ch);
			let outline = Self::create_outline(glyph_id, &mut scaler);
			let advance = Self::create_advance(font_ref, glyph_id, font_size);
			if let Some(out) = outline {
				let bounding_rect = Self::create_bounding_rect(&out);
				if !bounding_rect.size().is_empty() {
					let size = Self::create_texture_size(bounding_rect, padding);
					return VxTextBoundingSizeResult::new(ch, size, advance, Some(out))
				}
			}
			VxTextBoundingSizeResult::new(ch, VxSize::default(), advance, None)
		}).collect()
	}
}

pub(crate) struct VxTextBoundingSizeResult {
	pub ch: char,
	pub size: VxSize,
	pub advance: f32,
	pub outline: Option<Outline>,
}

impl VxTextBoundingSizeResult {
	pub fn new(ch: char, size: VxSize, advance: f32, outline: Option<Outline>) -> Self {
		Self { ch, size, advance, outline }
	}
}