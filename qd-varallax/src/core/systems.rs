use ahash::AHashMap;

use crate::{
	core::{
		glyph::{
			FALLBACK_FONT, FALLBACK_FONT_NAME, VxFont, VxFontAtlas, VxFontFamilyHash, VxGlyphInfo, VxVerticalMetrics
		},
		gpu_resource::{VxBindlessTextureModule, VxGpuResource, VxGpuTextureData},
		msdf::{VxFontDataGenerator, VxMsdfGenerator},
	},
	painter::{
		painter::{VxDrawTextData, VxVertexContainer},
		tessellate,
	},
	types::{
		color::VxColorU8,
		genelational_vector::{VxGenIndex, VxGenVector, VxSlot},
		geometry::{VxRect, VxSize, VxVec2},
		texture::{VxImage, VxTexture},
		vertex::VxTexVertex,
	},
};

pub struct VxTextureSystem {
	pub(crate) module: VxBindlessTextureModule,
	pub(crate) entries: VxGenVector<VxGpuTextureData>,
	pub(crate) is_registered: bool,
}

impl VxTextureSystem {
	pub(crate) fn new(gpu: &VxGpuResource) -> Self {
		Self {
			module: VxBindlessTextureModule::new(
				&gpu.device,
				&gpu.bind_group_layout,
				&gpu.sampler,
				wgpu::TextureFormat::Rgba8Unorm,
				VxGpuResource::TEXTURE_ARRAY_SIZE,
			),
			entries: VxGenVector::new(),
			is_registered: false,
		}
	}

	pub fn register_texture(&mut self, gpu: &VxGpuResource, texture: &mut VxTexture) {
		if texture.id().is_some() {
			return;
		}
		if let Some(img) = &mut texture.texture() {
			let wgpu_texture = VxBindlessTextureModule::create_texture(
				&gpu.device,
				wgpu::TextureFormat::Rgba8Unorm,
				"VxTexture",
				img.size(),
			);
			let wgpu_view = wgpu_texture.create_view(&wgpu::TextureViewDescriptor::default());

			gpu.queue.write_texture(
				wgpu_texture.as_image_copy(),
				img.as_raw_rgba8(),
				wgpu::TexelCopyBufferLayout {
					offset: 0,
					bytes_per_row: Some(img.size().width_u32() * 4),
					rows_per_image: None,
				},
				wgpu::Extent3d {
					width: img.size().width_u32(),
					height: img.size().height_u32(),
					depth_or_array_layers: 1,
				},
			);

			img.clear_pixels();

			let id = self
				.entries
				.insert(VxGpuTextureData::new(wgpu_view, wgpu_texture));
			texture.set_id(id);
			self.is_registered = true;
		}
	}

	// バインドグループを更新
	pub(crate) fn update_bind_group(&mut self, gpu: &VxGpuResource) {
		if !self.is_registered {
			return;
		}
		let texture_array_size = VxGpuResource::TEXTURE_ARRAY_SIZE as usize;
		let mut views_ref: Vec<&wgpu::TextureView> =
			vec![&self.module.dummy_view; texture_array_size];

		for (i, slot) in self.entries.slots.iter().enumerate() {
			if i >= texture_array_size {
				break;
			}
			if let VxSlot::Using { data, .. } = slot {
				views_ref[i] = &data.view;
			}
		}

		self.module.bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Vx Bindless Texture BindGroup"),
			layout: &gpu.bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureViewArray(&views_ref),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&gpu.sampler),
				},
			],
		});
		self.is_registered = false;
	}
}

// メモ!前提情報: アトラス上に複数種類のフォントを置くように作る。

pub(crate) struct VxFontSystem {
	// バインドレステクスチャモジュール
	pub module: VxBindlessTextureModule,
	// swashのScaleContext(使い回し用)
	pub context: swash::scale::ScaleContext,

	// VerticalMetricsデータ
	pub vertical_metrics: AHashMap<VxFontFamilyHash, VxVerticalMetrics>,
	// (フォントファミリー名ハッシュ, char), (アトラスID, グリフ情報)
	pub glyph_map: AHashMap<(VxFontFamilyHash, char), (VxGenIndex, VxGlyphInfo)>,
	// 実アトラス
	pub atlases: VxGenVector<(VxFontAtlas, VxGpuTextureData)>,
	// フォントの実バイナリデータ
	pub fonts_data: AHashMap<VxFontFamilyHash, Vec<u8>>,
}

impl VxFontSystem {
	pub const ATLAS_ARRAY_SIZE: u32 = 512;
	pub const ATLAS_SIZE: u32 = 2048;
	pub const MSDF_SIZE: f32 = 32.0;

	pub(crate) fn new(gpu: &VxGpuResource) -> Self {
		let mut sys = Self {
			module: VxBindlessTextureModule::new(
				&gpu.device,
				&gpu.bind_group_layout,
				&gpu.sampler,
				wgpu::TextureFormat::Rgba8Unorm,
				Self::ATLAS_ARRAY_SIZE,
			),
			context: swash::scale::ScaleContext::new(),
			vertical_metrics: AHashMap::new(),
			glyph_map: AHashMap::new(),
			atlases: VxGenVector::new(),
			fonts_data: AHashMap::new(),
		};
		sys.register_font(FALLBACK_FONT_NAME, FALLBACK_FONT);
		sys
	}

	pub fn register_font(&mut self, font_family: &str, font_data: &[u8]) {
		if let Some(font_ref) = VxFontDataGenerator::create_fontref(font_data) {
			let family = VxFont::hash(font_family);

			let vertical_metrics = VxFontDataGenerator::create_vertical_metrics(&font_ref);
			self.vertical_metrics.insert(family, vertical_metrics);

			self.fonts_data.insert(family, font_data.to_vec());
		}
	}

	pub(crate) fn ensure_glyphs(
		&mut self,
		gpu: &VxGpuResource,
		msdf_size: f32,
		mut font_family: VxFontFamilyHash,
		chars: &str,
	) -> VxFontFamilyHash {
		// フォントデータを特定、なければFALLBACKを使う
		let font_data: &[u8] = {
			if let Some(data) = self.fonts_data.get(&font_family) {
				data
			} else {
				font_family = VxFont::hash(FALLBACK_FONT_NAME);
				self.fonts_data.get(&font_family)
					.expect("VxFontSystem> CriticalError: Fallback font must be registered during system initalization.")
			}
		};

		// アトラスを取得
		let mut current_atlas_id = {
			if self.atlases.is_empty() {
				// ないから作る
				Self::create_new_atlas(&mut self.atlases, gpu)
			} else {
				self.atlases.last_id().unwrap()
			}
		};

		// 登録されていない文字を絞り出す
		let missings: Vec<char> = chars
			.chars()
			.filter(|c| !self.glyph_map.contains_key(&(font_family, *c)))
			.collect();

		// サイズを取得
		let mut size_result = VxFontDataGenerator::create_text_bounding_size(
			&mut self.context,
			font_data,
			msdf_size,
			VxMsdfGenerator::RANGE,
			&missings,
		);

		// 高さ優先でソート
		size_result.sort_unstable_by(|a, b| {
			b.size
				.height()
				.total_cmp(&a.size.height())
				.then_with(|| b.size.width().total_cmp(&a.size.width()))
		});

		for size_res in size_result.drain(..) {
			if let Some(shape) = size_res.shape {
				// パッカーに入れる
				let (target_id, rect) = self.prepare_atlas_space(current_atlas_id, size_res.size, gpu);
				current_atlas_id = target_id;

				let (atlas, wgpu_texture) = self.atlases.get_mut(current_atlas_id).unwrap();

				let (glyph_info, msdf_texture) = Self::create_glyph(rect, size_res.bounding_rect, shape, size_res.advance);

				self.glyph_map.insert((font_family, size_res.ch), (atlas.id, glyph_info));

				Self::update_atlas_texture(gpu, wgpu_texture, rect, msdf_texture);
			}
		}

		font_family
	}

	fn create_new_atlas(
		atlases: &mut VxGenVector<(VxFontAtlas, VxGpuTextureData)>,
		gpu: &VxGpuResource,
	) -> VxGenIndex {
		atlases.insert_with_key(|id| {
			let mut atlas = VxFontAtlas::new_empty(Self::ATLAS_SIZE);
			atlas.id = id;

			let texture = VxBindlessTextureModule::create_texture(
				&gpu.device,
				wgpu::TextureFormat::Rgba8Unorm,
				"VxFontAtlasTexture",
				VxSize::from_u32(Self::ATLAS_SIZE, Self::ATLAS_SIZE),
			);
			let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

			gpu.queue.write_texture(
				texture.as_image_copy(),
				VxImage::new(
					VxSize::from_u32(Self::ATLAS_SIZE, Self::ATLAS_SIZE),
					VxColorU8::from_hex(0x0000FF),
				)
				.as_raw_rgba8(),
				wgpu::TexelCopyBufferLayout {
					offset: 0,
					bytes_per_row: Some(Self::ATLAS_SIZE * 4),
					rows_per_image: None,
				},
				wgpu::Extent3d {
					width: Self::ATLAS_SIZE,
					height: Self::ATLAS_SIZE,
					depth_or_array_layers: 1,
				},
			);

			(atlas, VxGpuTextureData::new(view, texture))
		})
	}

	fn prepare_atlas_space(
		&mut self,
		mut atlas_id: VxGenIndex,
		size: VxSize,
		gpu: &VxGpuResource,
	) -> (VxGenIndex, VxRect) {
		debug_assert!(
			size.width_u32() <= Self::ATLAS_SIZE && size.height_u32() <= Self::ATLAS_SIZE,
			"VxFontSystem> Requested glyph size ({:?}) exceeds the atlas size ({:?}) and cannot be packed. Check the [msdf_size] argument.",
			size,
			Self::ATLAS_SIZE
		);

		loop {
			let (atlas, _) = self.atlases.get_mut(atlas_id).unwrap();
			if let Some(rect) = atlas.packer.insert(size) {
				return (atlas_id, rect);
			}

			atlas.is_full = true;
			atlas_id = Self::create_new_atlas(&mut self.atlases, gpu);
		}
	}

	fn create_glyph(
		rect: VxRect,
		bounding_rect: VxRect,
		shape: fdsm::shape::Shape<fdsm::shape::Contour>,
		advance: f32,
	) -> (VxGlyphInfo, VxImage) {
		// MSDFを生成
		let bearing_x = VxFontDataGenerator::create_bearing_x(bounding_rect, VxMsdfGenerator::RANGE);
		let bearing_y = VxFontDataGenerator::create_bearing_y(bounding_rect, VxMsdfGenerator::RANGE);

		let msdf_texture = VxMsdfGenerator::create_msdf_from_shape(bounding_rect, shape);
		let (w, h) = msdf_texture.size().to_tuple();

		let atlas_size = Self::ATLAS_SIZE as f32;

		let uv_rect = VxRect::new(
			rect.x() / atlas_size,
			rect.y() / atlas_size,
			w / atlas_size,
			h / atlas_size,
		);

		// グリフデータを作成
		let glyph_info = VxGlyphInfo::new(
			rect,
			uv_rect,
			bearing_x,
			bearing_y,
			advance,
			VxMsdfGenerator::RANGE as f32,
		);
		(glyph_info, msdf_texture)
	}

	fn update_atlas_texture(
		gpu: &VxGpuResource,
		wgpu_texture: &VxGpuTextureData,
		rect: VxRect,
		msdf_texture: VxImage,
	) {
		let (width, height) = msdf_texture.size().to_tuple_u32();

		gpu.queue.write_texture(
			wgpu::TexelCopyTextureInfo {
				texture: &wgpu_texture.texture,
				mip_level: 0,
				origin: wgpu::Origin3d {
					x: rect.pos().x_u32(),
					y: rect.pos().y_u32(),
					z: 0,
				},
				aspect: wgpu::TextureAspect::All,
			},
			msdf_texture.as_raw_rgba8(),
			wgpu::TexelCopyBufferLayout {
				offset: 0,
				bytes_per_row: Some(width * 4),
				rows_per_image: None,
			},
			wgpu::Extent3d {
				width,
				height,
				depth_or_array_layers: 1,
			},
		);
	}

	pub(crate) fn update_bind_group(&mut self, gpu: &VxGpuResource) {
		let texture_array_size = Self::ATLAS_ARRAY_SIZE as usize;
		let mut views_ref = vec![&self.module.dummy_view; texture_array_size];

		for (i, slot) in self.atlases.slots.iter().enumerate() {
			if i >= texture_array_size {
				break;
			}
			if let VxSlot::Using { data, .. } = slot {
				views_ref[i] = &data.1.view;
			}
		}

		self.module.bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Vx Bindless Texture BindGroup"),
			layout: &gpu.bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureViewArray(&views_ref),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&gpu.sampler),
				},
			],
		});
	}

	pub(crate) fn generate_text_verices(
		&mut self,
		gpu: &VxGpuResource,
		data: &[VxDrawTextData],
	) -> Vec<VxVertexContainer<VxTexVertex>> {
		let mut all_array = Vec::new();

		for cmd in data {
			let cmd_family = cmd.font.family();

			let resolved_family = self.ensure_glyphs(gpu, Self::MSDF_SIZE, cmd_family, &cmd.text);

			let font_size = cmd.font.pixel_size();
			let vertical_metrics = self.vertical_metrics.get(&resolved_family).expect(
				"VxFontSystem> CriticalError: Font metrics must exist for resolved font family.",
			);
			let line_height = vertical_metrics.create_line_height();
			let scale = font_size / Self::MSDF_SIZE;

			let mut cursor = VxVec2::default();

			for ch in cmd.text.chars() {
				let Some((atlas_and_texture_id, glyph_info)) = self.glyph_map.get(&(resolved_family, ch)) else { continue; };

				if ch == '\n' {
					cursor.set_x(0.0);
					cursor.set_y(cursor.y() + (line_height * scale));
					continue;
				}

				let x = cursor.x() + (glyph_info.bearing_x * scale);
				let y = cursor.y() + (glyph_info.bearing_y * scale);
				let w = glyph_info.atlas_rect.width() * scale;
				let h = glyph_info.atlas_rect.height() * scale;

				let (mut verts, indices) = tessellate::tessellate_texture(
					VxRect::new(x, y, w, h),
					cmd.color,
					glyph_info.uv_rect,
					atlas_and_texture_id.index as i32,
				);

				verts.iter_mut().for_each(|v| {
					let pos = cmd.matrix.transform_point(v.to_vec2());
					v.set_position_vec2(pos);
				});

				all_array.push(VxVertexContainer::new(verts.to_vec(), indices.to_vec()));

				cursor.set_x(cursor.x() + (glyph_info.advance * scale));
			}
		}

		self.update_bind_group(gpu);

		all_array
	}

	pub fn debug_atlas(&self, raw_atlas_index: i32) -> VxVertexContainer<VxTexVertex> {
		let (vert, index) = tessellate::tessellate_texture(
			VxRect::from_u32(0, 0, 2048, 2048), 0xFFFFFF.into(), VxRect::from_i32(0, 0, 1, 1), raw_atlas_index
		);
		VxVertexContainer::new(vert.to_vec(), index.to_vec())
	}
}
