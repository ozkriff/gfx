// Copyright 2015 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Raw Pipeline State Objects
//!
//! This module contains items used to create and manage a raw pipeline state object. Most users
//! will want to use the typed and safe `PipelineState`. See the `pso` module inside the `gfx`
//! crate.

use {MAX_COLOR_TARGETS, MAX_VERTEX_ATTRIBUTES, MAX_CONSTANT_BUFFERS,
     MAX_RESOURCE_VIEWS, MAX_UNORDERED_VIEWS, MAX_SAMPLERS};
use {ConstantBufferSlot, ColorSlot, ResourceViewSlot,
     UnorderedViewSlot, SamplerSlot,
     Primitive, Resources};
use {handle, format, state as s, texture};
use shade::Usage;
use std::error::Error;
use std::fmt;


/// Maximum number of vertex buffers used in a PSO definition.
pub const MAX_VERTEX_BUFFERS: usize = 4;

/// An offset inside a vertex buffer, in bytes.
pub type BufferOffset = usize;

/// Error types happening upon PSO creation on the device side.
#[derive(Clone, PartialEq, Debug)]
pub struct CreationError;

impl fmt::Display for CreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl Error for CreationError {
    fn description(&self) -> &str {
        "Could not create PSO on device."
    }
}

/// Color output configuration of the PSO.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ColorInfo {
    /// Color channel mask
    pub mask: s::ColorMask,
    /// Optional color blending
    pub color: Option<s::BlendChannel>,
    /// Optional alpha blending
    pub alpha: Option<s::BlendChannel>,
}
impl From<s::ColorMask> for ColorInfo {
    fn from(mask: s::ColorMask) -> ColorInfo {
        ColorInfo {
            mask: mask,
            color: None,
            alpha: None,
        }
    }
}
impl From<s::Blend> for ColorInfo {
    fn from(blend: s::Blend) -> ColorInfo {
        ColorInfo {
            mask: s::MASK_ALL,
            color: Some(blend.color),
            alpha: Some(blend.alpha),
        }
    }
}

/// Depth and stencil state of the PSO.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DepthStencilInfo {
    /// Optional depth test configuration
    pub depth: Option<s::Depth>,
    /// Optional stencil test on the front faces
    pub front: Option<s::StencilSide>,
    /// Optional stencil test on the back faces
    pub back: Option<s::StencilSide>,
}
impl From<s::Depth> for DepthStencilInfo {
    fn from(depth: s::Depth) -> DepthStencilInfo {
        DepthStencilInfo {
            depth: Some(depth),
            front: None,
            back: None,
        }
    }
}
impl From<s::Stencil> for DepthStencilInfo {
    fn from(stencil: s::Stencil) -> DepthStencilInfo {
        DepthStencilInfo {
            depth: None,
            front: Some(stencil.front),
            back: Some(stencil.back),
        }
    }
}
impl From<(s::Depth, s::Stencil)> for DepthStencilInfo {
    fn from(ds: (s::Depth, s::Stencil)) -> DepthStencilInfo {
        DepthStencilInfo {
            depth: Some(ds.0),
            front: Some(ds.1.front),
            back: Some(ds.1.back),
        }
    }
}

/// Index of a vertex buffer.
pub type BufferIndex = u8;
/// Offset of an attribute from the start of the buffer, in bytes
pub type ElemOffset = u32;
/// Offset between attribute values, in bytes
pub type ElemStride = u8;
/// The number of instances between each subsequent attribute value
pub type InstanceRate = u8;

/// A struct element descriptor.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Element<F> {
    /// Element format
    pub format: F,
    /// Offset from the beginning of the container, in bytes
    pub offset: ElemOffset,
}

/// Vertex buffer descriptor
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VertexBufferDesc {
    /// Total container size, in bytes
    pub stride: ElemStride,
    /// Rate of the input for the given buffer
    pub rate: InstanceRate,
}

/// PSO vertex attribute descriptor
pub type AttributeDesc = (BufferIndex, Element<format::Format>);
/// PSO constant buffer descriptor
pub type ConstantBufferDesc = Usage;
/// PSO shader resource view descriptor
pub type ResourceViewDesc = Usage;
/// PSO unordered access view descriptor
pub type UnorderedViewDesc = Usage;
/// PSO sampler descriptor
pub type SamplerDesc = Usage;
/// PSO color target descriptor
pub type ColorTargetDesc = (format::Format, ColorInfo);
/// PSO depth-stencil target descriptor
pub type DepthStencilDesc = (format::Format, DepthStencilInfo);

/// All the information surrounding a shader program that is required
/// for PSO creation, including the formats of vertex buffers and pixel targets;
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Descriptor {
    /// Type of the primitive
    pub primitive: Primitive,
    /// Rasterizer setup
    pub rasterizer: s::Rasterizer,
    /// Enable scissor test
    pub scissor: bool,
    /// Vertex buffers
    pub vertex_buffers: [Option<VertexBufferDesc>; MAX_VERTEX_BUFFERS],
    /// Vertex attributes
    pub attributes: [Option<AttributeDesc>; MAX_VERTEX_ATTRIBUTES],
    /// Constant buffers
    pub constant_buffers: [Option<ConstantBufferDesc>; MAX_CONSTANT_BUFFERS],
    /// Shader resource views
    pub resource_views: [Option<ResourceViewDesc>; MAX_RESOURCE_VIEWS],
    /// Unordered access views
    pub unordered_views: [Option<UnorderedViewDesc>; MAX_UNORDERED_VIEWS],
    /// Samplers
    pub samplers: [Option<SamplerDesc>; MAX_SAMPLERS],
    /// Render target views (RTV)
    pub color_targets: [Option<ColorTargetDesc>; MAX_COLOR_TARGETS],
    /// Depth stencil view (DSV)
    pub depth_stencil: Option<DepthStencilDesc>,
}

impl Descriptor {
    /// Create a new empty PSO descriptor.
    pub fn new(primitive: Primitive, rast: s::Rasterizer) -> Descriptor {
        Descriptor {
            primitive: primitive,
            rasterizer: rast,
            scissor: false,
            vertex_buffers: [None; MAX_VERTEX_BUFFERS],
            attributes: [None; MAX_VERTEX_ATTRIBUTES],
            constant_buffers: [None; MAX_CONSTANT_BUFFERS],
            resource_views: [None; MAX_RESOURCE_VIEWS],
            unordered_views: [None; MAX_UNORDERED_VIEWS],
            samplers: [None; MAX_SAMPLERS],
            color_targets: [None; MAX_COLOR_TARGETS],
            depth_stencil: None,
        }
    }
}

/// A complete set of vertex buffers to be used for vertex import in PSO.
#[derive(Copy, Clone, Debug)]
pub struct VertexBufferSet<R: Resources>(
    /// Array of buffer handles with offsets in them
    pub [Option<(R::Buffer, BufferOffset)>; MAX_VERTEX_ATTRIBUTES]
);

impl<R: Resources> VertexBufferSet<R> {
    /// Create an empty set
    pub fn new() -> VertexBufferSet<R> {
        VertexBufferSet([None; MAX_VERTEX_ATTRIBUTES])
    }
}

/// A constant buffer run-time parameter for PSO.
#[derive(Copy, Clone, Debug)]
pub struct ConstantBufferParam<R: Resources>(pub R::Buffer, pub Usage, pub ConstantBufferSlot);

/// A shader resource view (SRV) run-time parameter for PSO.
#[derive(Copy, Clone, Debug)]
pub struct ResourceViewParam<R: Resources>(pub R::ShaderResourceView, pub Usage, pub ResourceViewSlot);

/// An unordered access view (UAV) run-time parameter for PSO.
#[derive(Copy, Clone, Debug)]
pub struct UnorderedViewParam<R: Resources>(pub R::UnorderedAccessView, pub Usage, pub UnorderedViewSlot);

/// A sampler run-time parameter for PSO.
#[derive(Copy, Clone, Debug)]
pub struct SamplerParam<R: Resources>(pub R::Sampler, pub Usage, pub SamplerSlot);

/// A complete set of render targets to be used for pixel export in PSO.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct PixelTargetSet<R: Resources> {
    /// Array of color target views
    pub colors: [Option<R::RenderTargetView>; MAX_COLOR_TARGETS],
    /// Depth target view
    pub depth: Option<R::DepthStencilView>,
    /// Stencil target view
    pub stencil: Option<R::DepthStencilView>,
    /// Rendering dimensions
    pub size: texture::Dimensions,
}

impl<R: Resources> PixelTargetSet<R> {
    /// Create an empty set
    pub fn new() -> PixelTargetSet<R> {
        PixelTargetSet {
            colors: [None; MAX_COLOR_TARGETS],
            depth: None,
            stencil: None,
            size: (0, 0, 0, texture::AaMode::Single),
        }
    }
    /// Add a color view to the specified slot
    pub fn add_color(&mut self, slot: ColorSlot, view: &R::RenderTargetView,
                     dim: texture::Dimensions) {
        use std::cmp::max;
        self.colors[slot as usize] = Some(view.clone());
        self.size = max(self.size, dim);
    }
    /// Add a depth or stencil view to the specified slot
    pub fn add_depth_stencil(&mut self, view: &R::DepthStencilView,
                             has_depth: bool, has_stencil: bool,
                             dim: texture::Dimensions) {
        use std::cmp::max;
        if has_depth {
            self.depth = Some(view.clone());
        }
        if has_stencil {
            self.stencil = Some(view.clone());
        }
        self.size = max(self.size, dim);
    }
}

/// Informations about what is accessed by the pipeline
#[derive(Debug)]
pub struct AccessInfo<R: Resources> {
    mapped_reads: Vec<handle::RawMapping<R>>,
    mapped_writes: Vec<handle::RawMapping<R>>,
}

impl<R: Resources> AccessInfo<R> {
    /// Creates empty access informations
    pub fn new() -> Self {
        AccessInfo {
            mapped_reads: Vec::new(),
            mapped_writes: Vec::new(),
        }
    }

    /// Clear access informations
    pub fn clear(&mut self) {
        self.mapped_reads.clear();
        self.mapped_writes.clear();
    }

    /// Register a buffer read access
    pub fn buffer_read(&mut self, buffer: &handle::RawBuffer<R>) {
        if let Some(mapping) = buffer.mapping() {
            self.mapped_reads.push(mapping);
        }
    }

    /// Register a buffer write access
    pub fn buffer_write(&mut self, buffer: &handle::RawBuffer<R>) {
        if let Some(mapping) = buffer.mapping() {
            self.mapped_writes.push(mapping);
        }
    }

    /// Returns a slice of mappings associated to buffers that The GPU will read from
    pub fn mapped_reads(&self) -> &[handle::RawMapping<R>] {
        &self.mapped_reads[..]
    }

    /// Returns a slice of mappings associated to buffers that The GPU will write to
    pub fn mapped_writes(&self) -> &[handle::RawMapping<R>] {
        &self.mapped_writes[..]
    }
}

