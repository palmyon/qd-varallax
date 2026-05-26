use std::collections::HashMap;

use crate::{
	core::{
		glyph::{
			FALLBACK_FONT,
			VxAtlasRectResult,
			VxFont,
			VxFontAtlas,
			VxFontFamily,
			VxFontFamilyName
		},
		gpu_resource::{
			VxBindlessTextureModule,
			VxGpuResource,
			VxGpuTextureData
		},
		msdf::VxFontDataGenerator,
		resource::VxAppResources
	},
	painter::{
		painter::{
			VxDrawTextData,
			VxVertexContainer
		},
		tessellate
	},
	types::{
		genelational_vector::{
			VxGenIndex,
			VxSlot
		},
		geometry::{
			VxRect,
			VxSize,
			VxVec2
		},
		texture::{
			VxImage,
			VxTexture
		},
		vertex::VxTexVertex
	}
};


pub struct VxFontStorage {
	fonts: HashMap<VxFontFamilyName, Vec<u8>>,
}

impl VxFontStorage {
	pub fn new() -> Self {
		Self { fonts: HashMap::new() }
	}
	pub fn register_font(&mut self, family: impl Into<VxFontFamilyName>, data: &[u8]) {
		self.fonts.insert(family.into(), data.to_vec());
	}
	pub fn get_font_data(&self, family: &str) -> Option<&Vec<u8>> {
		self.fonts.get(family)
	}
	pub fn remove_font_data(&mut self, family: &str) -> bool {
		let res = self.fonts.remove(family);
		res.is_some()
	}
}

pub struct VxFontSystem {
	pub module: VxBindlessTextureModule,
	pub fonts: HashMap<VxFontFamilyName, VxFontFamily>,
	pub storage: VxFontStorage,
	pub context: swash::scale::ScaleContext,
}

impl VxFontSystem {
	pub const ATLAS_ARRAY_SIZE: u32 = 512;
	pub const ATLAS_SIZE: u32 = 2048;
	pub const MSDF_SIZE: f32 = 32.0;

	pub fn new(gpu: &VxGpuResource) -> Self {
		Self {
			module: VxBindlessTextureModule::new(
				&gpu.device,
				&gpu.bind_group_layout,
				&gpu.sampler,
				wgpu::TextureFormat::Rgba8Unorm,
				Self::ATLAS_ARRAY_SIZE,
			),
			fonts: HashMap::new(),
			storage: VxFontStorage::new(),
			context: swash::scale::ScaleContext::new(),
		}
	}

	pub fn set_init_glyphs(
		&mut self,
		gpu: &VxGpuResource,
		family: impl Into<VxFontFamilyName>,
		msdf_size: f32,
		chars: &str
	) {
		let result = self.ensure_glyphs(gpu, &family.into(), msdf_size, chars);

		for res in result {
			self.update_atlas_texture(gpu, res.atlas_id, res.rect, &res.msdf);
		}

		self.update_bind_group(gpu);
	}

	pub fn ensure_glyphs(
		&mut self,
		gpu: &VxGpuResource,
		family_name: &str,
		msdf_size: f32,
		chars: &str,
	) -> Vec<VxAtlasRectResult> {
		let font_data = self.storage.get_font_data(family_name)
			.map_or(FALLBACK_FONT, |v| v);

		let family = self.fonts.entry(family_name.to_string())
			.or_insert_with(|| {
				let font_ref = VxFontDataGenerator::create_fontref(font_data).unwrap();
				let vertical_metrics = VxFontDataGenerator::create_vertical_metrics(&font_ref);
				VxFontFamily::new(family_name, vertical_metrics)
			}
		);

		let mut missing_chars: Vec<char> = chars.chars()
			.filter(|&ch| !family.atlas_table.contains_key(&ch))
			.collect();

		if missing_chars.is_empty() { return vec![]; }
		let mut total_added_rect = vec![];

		while !missing_chars.is_empty() {
			let need_new_atlas = family.atlases.last().map_or(true, |a| a.is_full());
			if need_new_atlas {
				let mut new_atlas = VxFontAtlas::new_empty(family_name, Self::ATLAS_SIZE);
				new_atlas.is_registered = true;
				let data = Self::create_register_data(gpu, &mut new_atlas);
				self.module.entries.insert(data);
				let atlas_id = family.atlases.insert(new_atlas);
				let atlas_ref = family.atlases.get_mut(atlas_id).unwrap();
				atlas_ref.id = atlas_id;
			}

			let atlas = family.atlases.last_mut().unwrap();
			let current_missing_char: String = missing_chars.iter().collect();

			if let Some(result) = atlas.add_missing_chars(
				&mut self.context,
				font_data,
				&current_missing_char,
				msdf_size
			) {
				for res in result.added_rect {
					family.atlas_table.insert(res.ch, res.atlas_id);
					total_added_rect.push(res)
				}

				if let Some(failed) = result.failed_char {
					missing_chars = failed.into_iter().map(|f| f.ch).collect();
				} else {
					missing_chars.clear();
				}
			} else {
				break;
			}
		}
		total_added_rect
	}

	pub fn update_atlas_texture(
		&mut self,
		gpu: &VxGpuResource,
		id: VxGenIndex,
		rect: VxRect,
		msdf: &VxImage
	) {
		let Some(data) = self.module.entries.get(id) else { return; };

		let msdf_size = msdf.size();
		let width = msdf_size.width_u32();
		let height = msdf_size.height_u32();

		gpu.queue.write_texture(
			wgpu::TexelCopyTextureInfo {
				texture: &data.texture,
				mip_level: 0,
				origin: wgpu::Origin3d {
					x: rect.pos().x_u32(),
					y: rect.pos().y_u32(),
					z: 0,
				},
				aspect: wgpu::TextureAspect::All
			},
			msdf.as_raw_rgba8(),
			wgpu::TexelCopyBufferLayout {
				offset: 0,
				bytes_per_row: Some(width * 4),
				rows_per_image: None
			},
			wgpu::Extent3d {
				width,
				height,
				depth_or_array_layers: 1
			}
		);
	}

	pub(crate) fn update_bind_group(&mut self, gpu: &VxGpuResource) {
		let texture_array_size = Self::ATLAS_ARRAY_SIZE as usize;
		let mut views_ref = vec![&self.module.dummy_view; texture_array_size];

		for (i, slot) in self.module.entries.slots.iter().enumerate() {
			if i >= texture_array_size { break; }
			if let VxSlot::Using { data, .. } = slot {
				views_ref[i] = &data.view;
			}
		}

		self.module.bind_group = gpu.device.create_bind_group(
			&wgpu::BindGroupDescriptor {
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
			}
		);
	}

	pub(crate) fn embed_atlas_id(
		res: &mut VxAppResources,
		mut data: Vec<VxDrawTextData>,
	) -> Vec<VxVertexContainer<VxTexVertex>> {
		let mut all_array = vec![];

		for cmd in &data {
			let rect_result = res.fonts.ensure_glyphs(&res.gpu, cmd.font.family(), Self::MSDF_SIZE, &cmd.text);
			for rect in rect_result {
				res.fonts.update_atlas_texture(&res.gpu, rect.atlas_id, rect.rect, &rect.msdf);
			}
		}
		res.fonts.update_bind_group(&res.gpu);

		for cmd in data.drain(..) {
			let family_name = cmd.font.family();
			let Some(family) = res.fonts.get_atlas(family_name) else { continue; };
			let font_size = cmd.font.pixel_size();
			let line_height = family.vertical_metrics.create_line_height();
			let scale = font_size / Self::MSDF_SIZE;

			let mut cursor = VxVec2::default();

			for ch in cmd.text.chars() {
				let atlas_id = family.atlas_table.get(&ch);

				let glyph_info = atlas_id.and_then(|id| {
					let atlas = &family.atlases.get(*id)?;
					atlas.glyphs.get(&ch).map(|g| (id, g))
				});

				if let Some((id, glyph)) = glyph_info {
					if ch == '\n' {
						cursor.set_x(0.0);
						cursor.set_y(cursor.y() + (line_height * scale));
						continue;
					}

					let x = cursor.x() + (glyph.bearing_x * scale);
					let y = cursor.y() - (glyph.bearing_y * scale);
					let w = glyph.atlas_rect.width() * scale;
					let h = glyph.atlas_rect.height() * scale;

					let (mut verts, indices) = tessellate::tessellate_texture(
						VxRect::new(x, y, w, h),
						cmd.color,
						glyph.uv_rect,
						id.index as i32
					);

					for v in &mut verts {
						let pos = cmd.matrix.transform_point(v.to_vec2());
						v.set_position_vec2(pos);
					}

					all_array.push(VxVertexContainer::new(verts.to_vec(), indices.to_vec()));

					cursor.set_x(cursor.x() + (glyph.advance * scale));
				}
			}
		}

		all_array
	}

	pub fn create_text_bounding_rect(&self, text: &str, font: &VxFont) -> VxRect {
		let mut total_rect = VxRect::default();
		let mut base_pos = VxVec2::default();
		let scale = font.pixel_size() / Self::MSDF_SIZE;
		let Some(family) = self.get_atlas(font.family()) else { return VxRect::default(); };
		for ch in text.chars() {
			let Some(atlas_id) = family.atlas_table.get(&ch) else { return VxRect::default(); };
			let Some(glyph_info) = &family.atlases.get(*atlas_id).unwrap().glyphs.get(&ch) else { return VxRect::default(); };
			total_rect = total_rect.union(
				glyph_info.atlas_rect
				.with_pos(base_pos)
			);
			base_pos.set_x(base_pos.x() +( glyph_info.advance));
		}
		total_rect
	}

	fn get_atlas(&self, family: &str) -> Option<&VxFontFamily> {
		self.fonts.get(family)
	}

	fn create_register_data(gpu: &VxGpuResource, atlas: &mut VxFontAtlas) -> VxGpuTextureData {
		let texture = VxBindlessTextureModule::create_texture(
			&gpu.device,
			wgpu::TextureFormat::Rgba8Unorm,
			"VxAtlas",
			VxSize::from_u32(Self::ATLAS_SIZE, Self::ATLAS_SIZE),
		);
		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

		gpu.queue.write_texture(
			texture.as_image_copy(),
			&atlas.texture_data.to_raw_rgba8(),
			wgpu::TexelCopyBufferLayout {
				offset: 0,
				bytes_per_row: Some(Self::ATLAS_SIZE * 4),
				rows_per_image: None,
			},
			wgpu::Extent3d { width: Self::ATLAS_SIZE, height: Self::ATLAS_SIZE, depth_or_array_layers: 1 },
		);

		atlas.texture_data.clear_pixels();

		VxGpuTextureData::new(view, texture)
	}
}

pub struct VxTextureSystem {
	pub(crate) module: VxBindlessTextureModule,
	pub(crate) is_registered: bool,
}

impl VxTextureSystem {
	pub fn new(gpu: &VxGpuResource) -> Self {
		Self {
			module: VxBindlessTextureModule::new(
				&gpu.device,
				&gpu.bind_group_layout,
				&gpu.sampler,
				wgpu::TextureFormat::Rgba8Unorm,
				VxGpuResource::TEXTURE_ARRAY_SIZE,
			),
			is_registered: false,
		}
	}

	pub fn register_texture(&mut self, gpu: &VxGpuResource, texture: &mut VxTexture) {
		if texture.id().is_some() { return; }
		if let Some(img) = &mut texture.texture() {
			let wgpu_texture = VxBindlessTextureModule::create_texture(
				&gpu.device, wgpu::TextureFormat::Rgba8Unorm, "VxTexture", img.size()
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
				wgpu::Extent3d { width: img.size().width_u32(), height: img.size().height_u32(), depth_or_array_layers: 1 }
			);

			img.clear_pixels();

			let id = self.module.entries.insert(VxGpuTextureData::new(wgpu_view, wgpu_texture));
			texture.set_id(id);
			self.is_registered = true;
		}
	}

	// バインドグループを更新
	pub(crate) fn update_bind_group(&mut self, gpu: &VxGpuResource) {
		if !self.is_registered { return; }
		let texture_array_size = VxGpuResource::TEXTURE_ARRAY_SIZE as usize;
		let mut views_ref: Vec<&wgpu::TextureView>  = vec![&self.module.dummy_view; texture_array_size];

		for (i, slot) in self.module.entries.slots.iter().enumerate() {
			if i >= texture_array_size { break; }
			if let VxSlot::Using { data, .. } = slot {
				views_ref[i] = &data.view;
			}
		}

		self.module.bind_group = gpu.device.create_bind_group(
			&wgpu::BindGroupDescriptor {
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
			}
		);
		self.is_registered = false;
	}
}