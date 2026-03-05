use iced::advanced::graphics;
use iced::wgpu;
use iced::wgpu::{wgt, Device, DeviceDescriptor, GlBackendOptions, Texture};
use iced::wgpu::hal::gles;
use iced::wgpu::hal::gles::{TextureFormatDesc, TextureInner};
use iced_core::{Color, Font, Pixels, Size};
use iced_graphics::{Shell, Viewport};
use iced_wgpu::Engine;
use iced_wgpu::wgpu::Gles3MinorVersion;
use iced_wgpu::wgpu::hal::Adapter;
use log::info;
use crate::rg35xxp::egl;
use crate::rg35xxp::egl::FramebufferWindow;

// 一个简单的 WGSL Shader，用于翻转 Y 轴
const BLIT_SHADER: &str = include_str!("blit.glsl");
pub(crate) struct Wrap {
    viewport: Viewport,
    surface_texture: Texture,

    // 离屏渲染，然后翻转一次
    offscreen_texture: Texture,
    offscreen_view: wgpu::TextureView,
    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group: wgpu::BindGroup,

    // 渲染器最后回收
    renderer: iced_wgpu::Renderer,
    queue: wgpu::Queue,
    device: Device,
    framebuffer: FramebufferWindow,
}
impl Wrap {
    pub fn from(framebuffer: FramebufferWindow) -> Result<Self, &'static str> {
        unsafe { Self::from_unsafe(framebuffer) }
    }
    unsafe fn get_default_texture(
        fb: &FramebufferWindow,
        device: &wgpu::Device,
    ) -> Result<Texture, &'static str> {
        let (width, height) = fb.size();

        const GL_RGBA: u32 = 0x1908;
        const GL_RGBA8: u32 = 0x8058;
        const GL_UNSIGNED_BYTE: u32 = 0x1401;
        let hal_texture = gles::Texture {
            inner: TextureInner::DefaultRenderbuffer, // 核心：你的 GL 纹理 ID
            drop_guard: None,                         // 我们自己管理生命周期，不让 wgpu 删除它
            mip_level_count: 1,
            array_layer_count: 1,
            format: wgpu::TextureFormat::Rgba8Unorm,
            format_desc: TextureFormatDesc {
                internal: GL_RGBA8,          // 对应 Rgba8Unorm 的内部格式
                external: GL_RGBA,           // 外部格式
                data_type: GL_UNSIGNED_BYTE, // 数据类型
            },
            copy_size: wgpu::hal::CopyExtent {
                width,
                height,
                depth: 1,
            },
        };

        // 3. 定义 wgpu 纹理描述符
        let descriptor = wgpu::TextureDescriptor {
            label: Some("Imported GL Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };

        info!("try to create texture");
        Ok(unsafe { device.create_texture_from_hal::<gles::Api>(hal_texture, &descriptor) })
    }
    // 初始化中间纹理和 Blit 管道
    fn init_blit(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> (
        Texture,
        wgpu::TextureView,
        wgpu::RenderPipeline,
        wgpu::BindGroup,
    ) {
        // 1. 创建中间纹理 (Iced 将渲染到这里)
        let texture_desc = wgpu::TextureDescriptor {
            label: Some("Offscreen Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format, // 通常是 Rgba8Unorm
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let offscreen_texture = device.create_texture(&texture_desc);
        let offscreen_view = offscreen_texture.create_view(&Default::default());

        // 2. 创建采样器
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // 3. 编译 Shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Blit Shader"),
            source: wgpu::ShaderSource::Wgsl(BLIT_SHADER.into()),
        });

        // 4. 创建 Pipeline
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blit Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blit Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Blit Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None, // <--- 关键：设置为 None，确保正反面都渲染，避免翻车
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        // 5. 创建 BindGroup
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&offscreen_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("Blit Bind Group"),
        });

        // 这里的 offscreen_texture 需要包装进 iced_wgpu::wgpu::Texture 结构才能返回给 Wrap 使用吗？
        // 实际上 iced 的 Texture 只是 wgpu::Texture 的 re-export
        (offscreen_texture, offscreen_view, pipeline, bind_group)
    }
    unsafe fn from_unsafe(framebuffer: FramebufferWindow) -> Result<Self, &'static str> { unsafe {
        framebuffer.make_current()?;
        let Some(hal_adapter) =
            gles::Adapter::new_external(egl::get_proc_address, GlBackendOptions{ gles_minor_version: Gles3MinorVersion::Version2, fence_behavior: Default::default() })
        else {
            return Err("Failed to create HAL adapter from external GL");
        };

        let Ok(hal_device) = hal_adapter.adapter.open(
            wgt::Features::empty(),
            &wgpu::Limits::default(),
            &wgt::MemoryHints::default(),
        ) else {
            return Err("Failed to open HAL device");
        };

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let adapter = instance.create_adapter_from_hal(hal_adapter);

        let descriptor = DeviceDescriptor::default();
        let Ok((device, queue)) =
            adapter.create_device_from_hal::<gles::Api>(hal_device, &descriptor)
        else {
            return Err("Failed to create wgpu device from HAL");
        };
        let surface_texture = Self::get_default_texture(&framebuffer, &device)?;
        let (width, height) = framebuffer.size();
        let viewport = Viewport::with_physical_size(Size::new(width, height), 1.0);
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let (offscreen_texture, offscreen_view, blit_pipeline, blit_bind_group) =
            Self::init_blit(&device, format, width as u32, height as u32);

        let engine = Engine::new(
            &adapter,
            device.clone(),
            queue.clone(),
            wgpu::TextureFormat::Rgba8Unorm,
            Some(graphics::Antialiasing::MSAAx2),
            Shell::headless(),
        );
        let default_font = Font::DEFAULT;
        let default_text_size = Pixels(16.0);
        let renderer = iced_wgpu::Renderer::new(engine, default_font, default_text_size);
        Ok(Self {
            framebuffer,
            device,
            queue,
            renderer,
            surface_texture,
            viewport,
            offscreen_texture,
            offscreen_view,
            blit_pipeline,
            blit_bind_group,
        })
    }}

    pub fn renderer(&mut self) -> &mut iced_wgpu::Renderer {
        &mut self.renderer
    }
    pub fn size(&self) -> (u32, u32) {
        self.framebuffer.size()
    }
    pub fn present(&mut self) {
        self.renderer.present(
            Some(Color::WHITE),
            wgpu::TextureFormat::Rgba8Unorm,
            &self.offscreen_view,
            &self.viewport,
        );
        let surface_view = self.surface_texture.create_view(&Default::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Blit Encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Blit Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), // 背景色
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.blit_pipeline);
            pass.set_bind_group(0, &self.blit_bind_group, &[]);
            pass.draw(0..3, 0..1); // 必须是 0..3，对应 shader 里的 array 大小
        }

        self.queue.submit(Some(encoder.finish()));
        self.framebuffer.check_error();
        self.framebuffer.present();
    }
}

