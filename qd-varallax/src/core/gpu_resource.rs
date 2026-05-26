
use crate::types::{
	genelational_vector::VxGenVector, geometry::VxSize
	};

pub(crate) struct VxGpuTextureData {
	pub view: wgpu::TextureView,
	pub texture: wgpu::Texture,
}

impl VxGpuTextureData {
	pub fn new(view: wgpu::TextureView, texture: wgpu::Texture) -> Self {
		Self { view, texture }
	}
}

pub(crate) struct VxBindlessTextureModule {
	pub(crate) bind_group: wgpu::BindGroup,
	pub(crate) dummy_view: wgpu::TextureView,
	pub(crate) entries: VxGenVector<VxGpuTextureData>,
}

impl VxBindlessTextureModule {
	pub(crate) fn new(
		device: &wgpu::Device,
		bind_group_layout: &wgpu::BindGroupLayout,
		sampler: &wgpu::Sampler,
		texture_format: wgpu::TextureFormat,
		array_size: u32,
	) -> Self {
		let dummy_texture = device.create_texture(
			&wgpu::TextureDescriptor {
				label: Some("Vx Dummy Texture"),
				size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
				mip_level_count: 1,
				sample_count: 1,
				dimension: wgpu::TextureDimension::D2,
				format: texture_format,
				usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
				view_formats: &[],
			}
		);

		let dummy_view = dummy_texture.create_view(&wgpu::TextureViewDescriptor::default());
		let views_ref = vec![&dummy_view; array_size as usize];

		let bind_group = device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				label: Some("Vx Bindless Texture BindGroup"),
				layout: &bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureViewArray(&views_ref),
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&sampler),
					},
				],
			}
		);

		Self {
			bind_group,
			dummy_view,
			entries: VxGenVector::new(),
		}
	}

	pub fn create_texture(
		device: &wgpu::Device,
		format: wgpu::TextureFormat,
		label: impl Into<String>,
		size: VxSize,
	) -> wgpu::Texture {
		device.create_texture(
			&wgpu::TextureDescriptor {
				label: Some(&label.into()),
				size: wgpu::Extent3d { width: size.width_u32(), height: size.height_u32(), depth_or_array_layers: 1 },
				mip_level_count: 1,
				sample_count: 1,
				dimension: wgpu::TextureDimension::D2,
				format,
				usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
				view_formats: &[],
			}
		)
	}
}



pub struct VxGpuResource {
	pub(crate) instance: wgpu::Instance,
	pub(crate) adapter: wgpu::Adapter,
	pub(crate) device: wgpu::Device,
	pub(crate) queue: wgpu::Queue,

	// テクスチャ用
	pub(crate) bind_group_layout: wgpu::BindGroupLayout,
	pub(crate) sampler: wgpu::Sampler,
}

impl VxGpuResource {
	pub const TEXTURE_ARRAY_SIZE: u32 = 512;
	pub(crate) async fn new() -> Self {
		let instance = wgpu::Instance::default();

		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions::default())
			.await
			.unwrap();

		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: None,
					required_features: wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
									| wgpu::Features::TEXTURE_BINDING_ARRAY,
					required_limits: wgpu::Limits {
						max_binding_array_elements_per_shader_stage: Self::TEXTURE_ARRAY_SIZE,
						..Default::default()
					},
					..Default::default()
				},
			)
			.await
			.unwrap();

		// テクスチャバインドグループレイアウトの作成
		let bind_group_layout = device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				label: Some("Vx Bindless Texture Layout"),
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							sample_type: wgpu::TextureSampleType::Float { filterable: true },
							view_dimension: wgpu::TextureViewDimension::D2,
							multisampled: false,
						},
						count: std::num::NonZeroU32::new(Self::TEXTURE_ARRAY_SIZE),
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None,
					},
				],
			},
		);

		let sampler = device.create_sampler(
			&wgpu::SamplerDescriptor {
				label: Some("Vx Commom Sampler"),
				address_mode_u: wgpu::AddressMode::ClampToEdge,
				address_mode_v: wgpu::AddressMode::ClampToEdge,
				mag_filter: wgpu::FilterMode::Linear,
				min_filter: wgpu::FilterMode::Linear,
				..Default::default()
			}
		);

		// 仮書き込み(画像0の時のお祈り用)
		let dummy_texture = device.create_texture(
			&wgpu::TextureDescriptor {
				label: Some("Vx Dummy Texture"),
				size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
				mip_level_count: 1,
				sample_count: 1,
				dimension: wgpu::TextureDimension::D2,
				format: wgpu::TextureFormat::Rgba8Unorm,
				usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
				view_formats: &[],
			}
		);
		queue.write_texture(
			dummy_texture.as_image_copy(),
			&[255, 0, 255, 255],
			wgpu::TexelCopyBufferLayout {
				offset: 0,
				bytes_per_row: Some(4),
				rows_per_image: None,
			},
			wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
		);

		Self {
			instance,
			adapter,
			device,
			queue,
			bind_group_layout,
			sampler,
		}
	}

	pub(crate) fn update_surface_config(&self, surface: &wgpu::Surface, config: &wgpu::SurfaceConfiguration) {
		surface.configure(&self.device, config);
	}
}