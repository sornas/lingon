//! Puts pixels on your screen.
//!
//! A basic example:
//! ```ignore
//! let mut renderer = Renderer::new(&mut context, sampler);
//! loop {
//!     renderer.push(Rect::new()
//!         .at(0.5, 0.1)
//!         .angle(0.5)
//!         .scale(0.1, 2.0)
//!     );
//!
//!     renderer.render(&mut context).unwrap();
//! }
//! ```
//!
//! Here we instance a Rect, and then place it using some
//! convenience functions. We then tell the renderer to render it.
//!
//! Since the renderer is based around
//! [instancing](https://www.khronos.org/opengl/wiki/Vertex_Rendering#Instancing),
//! some things (like skewing) are harder to do.

pub use crate::renderer::particles::ParticleSystem;

use crate::asset::{Image, Pixels};
use crate::renderer::particles::FrozenParticles;

use cgmath::Vector2;
use luminance::{blending::{Blending, Equation, Factor}, context::GraphicsContext};
use luminance::pipeline::PipelineState;
use luminance::pixel::NormRGBA8UI;
use luminance::render_state::RenderState;
use luminance::tess::Mode;
use luminance::texture::{Dim3, GenMipmaps, Sampler, Texture};
use luminance_sdl2::GL33Surface;

pub mod particles;
mod prelude;

// Me no likey, but at least it's not documented.
use crate::renderer::prelude::*;

pub type SpriteSheetID = usize;

/// Type used to simplify some types.
pub type Tex = Texture<<GL33Surface as GraphicsContext>::Backend, Dim3, NormRGBA8UI>;

/// Vertex shader source code.
const VS_STR: &str = include_str!("vs.glsl");
/// Fragment shader source code.
const FS_STR: &str = include_str!("fs.glsl");
/// Particle vertex shader source code.
const VS_PARTICLE_STR: &str = include_str!("vs_particle.glsl");
/// The maximum size of a sprite sheet, and the maximum number of
/// sprite sheets.
const SPRITE_SHEET_SIZE: [u32; 3] = [512, 512, 512];

/// A simple rectangle for rendering sprites and the like.
const RECT: [Vertex; 6] = [
    Vertex::new(VPosition::new([-0.5, -0.5])),
    Vertex::new(VPosition::new([0.5, -0.5])),
    Vertex::new(VPosition::new([0.5, 0.5])),
    Vertex::new(VPosition::new([0.5, 0.5])),
    Vertex::new(VPosition::new([-0.5, 0.5])),
    Vertex::new(VPosition::new([-0.5, -0.5])),
];

/// A sprite sheet that lives on the GPU.
#[derive(Clone, Debug)]
pub struct SpriteSheet {
    id: usize,
    image: Image,
    tile_size: (Pixels, Pixels),
}

impl SpriteSheet {
    /// Returns the SpriteRegion of a tile given the specified tile sizes,
    /// starting from the top left.
    pub fn grid(&self, tx: usize, ty: usize) -> SpriteRegion {
        let xlo = ((self.tile_size.0 * tx) as f32) / (SPRITE_SHEET_SIZE[0] as f32);
        let ylo = ((self.tile_size.1 * ty) as f32) / (SPRITE_SHEET_SIZE[1] as f32);
        let w = (self.tile_size.0 as f32) / (SPRITE_SHEET_SIZE[0] as f32);
        let h = (self.tile_size.1 as f32) / (SPRITE_SHEET_SIZE[1] as f32);
        (
            self.id as f32 / (SPRITE_SHEET_SIZE[2] as f32),
            [xlo, ylo, xlo + w, ylo + h],
        )
    }

    pub fn upload(&self, tex: &mut Tex) {
        tex.upload_part_raw(
            GenMipmaps::No,
            [0, 0, self.id as u32],
            [self.image.width as u32, self.image.height as u32, 1],
            &self.image.texture_data,
        )
        .unwrap();
    }

    pub fn reload(&mut self, tex: &mut Tex) {
        if self.image.reload() {
            self.upload(tex);
        }
    }
}

// Helper macro for fast writing of boilerplate code.
macro_rules! impl_transform {
    (deref, $fn:ident, $op:tt, $( $var:ident : $type:ident => $set:tt ),*) => {
        fn $fn(&mut self, $( $var : $type ),*) -> &mut Self {
            $(
                *self.$set() $op $var;
            )*
            self
        }
    };

    (arr, $fn:ident, $op:tt, $( $var:ident : $type:ident => $arr:tt [ $idx:expr ] ),*) => {
        fn $fn(&mut self, $( $var : $type ),*) -> &mut Self {
            $(
                self.$arr()[$idx] $op $var;
            )*
            self
        }
    };
}

// A fast and hacky way to implement the Transform trait when
// the fields are named the same. Maybe make it a bit more robust?
macro_rules! impl_transform_for {
    ($ty:ident) => {
        impl Transform for $ty {
            fn x_mut(&mut self) -> &mut f32 {
                &mut self.position.x
            }
            fn y_mut(&mut self) -> &mut f32 {
                &mut self.position.y
            }

            fn sx_mut(&mut self) -> &mut f32 {
                &mut self.scale.x
            }

            fn sy_mut(&mut self) -> &mut f32 {
                &mut self.scale.y
            }

            fn r_mut(&mut self) -> &mut f32 {
                &mut self.rotation
            }
        }
    };
}

/// Manipulate and move things around.
/// Designed to be chainable.
pub trait Transform {
    /// The x-component of the position.
    fn x_mut(&mut self) -> &mut f32;
    /// The y-component of the position.
    fn y_mut(&mut self) -> &mut f32;

    /// The x-component of the scale.
    fn sx_mut(&mut self) -> &mut f32;
    /// The y-component of the scale.
    fn sy_mut(&mut self) -> &mut f32;

    /// The rotation.
    fn r_mut(&mut self) -> &mut f32;

    // TODO(ed): Comment on the functions the macro implement?
    impl_transform!(deref, move_by,  +=, x:  f32 => x_mut,         y:  f32 => y_mut);
    impl_transform!(deref, at,       =,  x:  f32 => x_mut,         y:  f32 => y_mut);
    impl_transform!(deref, angle,    =,  r:  f32 => r_mut);
    impl_transform!(deref, rotate,   +=, r:  f32 => r_mut);
    impl_transform!(deref, scale_by, *=, sx: f32 => sx_mut,        sy: f32 => sy_mut);
    impl_transform!(deref, scale,     =, sx: f32 => sx_mut,        sy: f32 => sy_mut);
}

/// Colorable things are Tint-able!
pub trait Tint {
    fn color_mut(&mut self) -> &mut [f32; 4];

    // TODO(ed): Comment on the functions the macro implement?
    impl_transform!(arr, rgb,  =,  r:  f32 => color_mut[0],  g:  f32 => color_mut[1], b: f32 => color_mut[2]);
    impl_transform!(arr, rgba, *=, r:  f32 => color_mut[0],  g:  f32 => color_mut[1], b: f32 => color_mut[2], a: f32 => color_mut[3]);
    impl_transform!(arr, r,    *=, r:  f32 => color_mut[0]);
    impl_transform!(arr, g,    *=, g:  f32 => color_mut[1]);
    impl_transform!(arr, b,    *=, b:  f32 => color_mut[2]);
    impl_transform!(arr, a,    *=, a:  f32 => color_mut[3]);

    fn tint(&mut self, r: f32, g: f32, b: f32, a: f32) -> &mut Self {
        self.rgba(r, g, b, a)
    }
}

/// A function that renders.
pub type RenderFn = dyn FnMut(
    &[Instance],
    &[FrozenParticles],
    &mut Tex,
    cgmath::Matrix4<f32>,
    &mut GL33Surface,
) -> Result<(), ()>;

/// From where you see the world. Can be moved around via [Transform].
pub struct Camera {
    position: Vector2<f32>,
    scale: Vector2<f32>,
    rotation: f32,
}

impl_transform_for!(Camera);

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Vector2::new(0.0, 0.0),
            scale: Vector2::new(1.0, 1.0),
            rotation: 0.0,
        }
    }

    /// Converts the camera to a matrix for sending to the GPU.
    pub fn matrix(&self) -> cgmath::Matrix4<f32> {
        use cgmath::{Matrix4, Rad, Vector3};
        let scale = Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, 0.0);
        let rotation = Matrix4::from_angle_z(Rad(self.rotation));
        let translation =
            Matrix4::from_translation(Vector3::new(self.position.x, self.position.y, 0.0));
        scale * rotation * translation
    }
}

/// A big struct holding all the rendering state.
pub struct Renderer {
    pub camera: Camera,
    pub render_fn: Box<RenderFn>,
    pub instances: Vec<Instance>,
    pub particles: Vec<FrozenParticles>,
    pub tex: Tex,
    pub sprite_sheets: Vec<SpriteSheet>,
}

/// If something can be rendered, it has to be Stamp.
pub trait Stamp {
    fn stamp(self) -> Instance;
}

/// A rectangle that can be rendered to the screen.
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    position: Vector2<f32>,
    scale: Vector2<f32>,
    rotation: f32,
    color: [f32; 4],
}

impl_transform_for!(Rect);

impl Tint for Rect {
    fn color_mut(&mut self) -> &mut [f32; 4] {
        &mut self.color
    }
}

impl Stamp for &Rect {
    fn stamp(self) -> Instance {
        Instance {
            position: IPosition::new(self.position.into()),
            rotation: IRotation::new(self.rotation),
            scale: IScale::new(self.scale.into()),
            color: IColor::new(self.color),
            sheet: ISheet::new(-1.0),
            uv: IUV::new([0.0, 0.0, 1.0, 1.0]),
        }
    }
}

impl Stamp for Rect {
    fn stamp(self) -> Instance {
        (&self).stamp()
    }
}

// TODO(ed): This feels dumb...
impl Stamp for &mut Rect {
    fn stamp(self) -> Instance {
        (*self).stamp()
    }
}

#[allow(dead_code)]
impl Rect {
    pub fn new() -> Self {
        Self {
            position: Vector2::new(0.0, 0.0),
            scale: Vector2::new(1.0, 1.0),
            rotation: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// A rectangle that has a nice image on it.
#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    position: Vector2<f32>,
    scale: Vector2<f32>,
    rotation: f32,
    color: [f32; 4],
    sheet: f32,
    rect: [f32; 4],
}

impl_transform_for!(Sprite);

impl Tint for Sprite {
    fn color_mut(&mut self) -> &mut [f32; 4] {
        &mut self.color
    }
}

impl Stamp for &Sprite {
    fn stamp(self) -> Instance {
        Instance {
            position: IPosition::new(self.position.into()),
            rotation: IRotation::new(self.rotation),
            scale: IScale::new(self.scale.into()),
            color: IColor::new(self.color),
            sheet: ISheet::new(self.sheet),
            uv: IUV::new(self.rect),
        }
    }
}

impl Stamp for Sprite {
    fn stamp(self) -> Instance {
        (&self).stamp()
    }
}

impl Stamp for &mut Sprite {
    fn stamp(self) -> Instance {
        (*self).stamp()
    }
}

/// A piece of a SpriteSheet to render.
type SpriteRegion = (f32, [f32; 4]);

impl Sprite {
    pub fn new(region: SpriteRegion) -> Self {
        Self {
            position: Vector2::new(0.0, 0.0),
            scale: Vector2::new(1.0, 1.0),
            rotation: 0.0,
            color: [1.0, 1.0, 1.0, 1.0],
            sheet: region.0,
            rect: region.1,
        }
    }
}

impl Renderer {
    /// Create a new Render instance.
    pub fn new(context: &mut GL33Surface, sampler: Sampler) -> Self {
        let back_buffer = context.back_buffer().unwrap();

        // Setup shader programs.

        let mut sprite_program = context
            .new_shader_program::<VertexSemantics, (), ShaderInterface>()
            .from_strings(VS_STR, None, None, FS_STR)
            .unwrap()
            .ignore_warnings();

        let mut particle_program = context
            .new_shader_program::<VertexSemantics, (), ShaderInterface>()
            .from_strings(VS_PARTICLE_STR, None, None, FS_STR)
            .unwrap()
            .ignore_warnings();

        let tex: Tex =
            Texture::new(context, SPRITE_SHEET_SIZE, 0, sampler).expect("failed to create texture");

        // This function draws the entire frame and is called continuously.
        let render_fn = move |instances: &[Instance],
                              systems: &[FrozenParticles],
                              tex: &mut Tex,
                              view: cgmath::Matrix4<f32>,
                              context: &mut GL33Surface| {
            let triangle = context
                .new_tess()
                .set_vertices(&RECT[..])
                .set_instances(&instances[..])
                .set_mode(Mode::Triangle)
                .build()
                .unwrap();

            let particles: Vec<_> = systems
                .iter()
                .map(|s| {
                    (
                        s.time,
                        context
                            .new_tess()
                            .set_vertices(&RECT[..])
                            .set_instances(&s.particles[..])
                            .set_mode(Mode::Triangle)
                            .build()
                            .unwrap(),
                    )
                })
                .collect();

            let render = context
                .new_pipeline_gate()
                .pipeline(
                    &back_buffer,
                    &PipelineState::default(),
                    |pipeline, mut shd_gate| {
                        let bound_tex = pipeline.bind_texture(tex)?;

                        let state = RenderState::default().set_depth_test(None).set_blending(Blending {
                            equation: Equation::Additive,
                            src: Factor::SrcAlpha,
                            dst: Factor::SrcAlphaComplement,
                        });
                        shd_gate.shade(&mut sprite_program, |mut iface, uni, mut rdr_gate| {
                            iface.set(&uni.tex, bound_tex.binding());
                            iface.set(&uni.view, view.into());

                            rdr_gate.render(&state, |mut tess_gate| tess_gate.render(&triangle))
                        })?;

                        shd_gate.shade(&mut particle_program, |mut iface, uni, mut rdr_gate| {
                            iface.set(&uni.tex, bound_tex.binding());
                            iface.set(&uni.view, view.into());
                            rdr_gate.render(&state, |mut tess_gate| {
                                for (t, p) in particles {
                                    iface.set(&uni.t, t);
                                    tess_gate.render(&p)?;
                                }
                                Ok(())
                            })
                        })
                    },
                )
                .assume();

            if render.is_ok() {
                context.window().gl_swap_window();
                Ok(())
            } else {
                Err(())
            }
        };

        Self {
            camera: Camera::new(),
            render_fn: Box::new(render_fn),
            instances: Vec::new(),
            tex,
            sprite_sheets: Vec::new(),
            particles: Vec::new(),
        }
    }

    /// Queues the stamp for rendering.
    pub fn push<T: Stamp>(&mut self, stamp: T) {
        self.instances.push(stamp.stamp());
    }

    /// Queues the particle_systems for rendering.
    pub fn push_particle_system(&mut self, system: &ParticleSystem) {
        self.particles.push(system.freeze());
    }

    /// Registers an image as a new sprite sheet with the specified tile size.
    ///
    /// There's a hard limit on the number of SpriteSheets that can be
    /// added: see [SPRITE_SHEET_SIZE].
    pub fn add_sprite_sheet(&mut self, image: Image, tile_size: (Pixels, Pixels)) -> SpriteSheetID {
        let id = self.sprite_sheets.len();
        assert!((id as u32) < SPRITE_SHEET_SIZE[2]);

        // Upload texture to slot
        let sheet = SpriteSheet {
            id,
            image,
            tile_size,
        };
        sheet.upload(&mut self.tex);
        self.sprite_sheets.push(sheet);
        id
    }

    /// Reload all assets that the renderer owns.
    ///
    /// Currently this means as sprite sheets.
    pub fn reload(&mut self) {
        for sheet in self.sprite_sheets.iter_mut() {
            sheet.reload(&mut self.tex);
        }
    }

    pub fn render(&mut self, context: &mut GL33Surface) -> Result<(), ()> {
        let res = (*self.render_fn)(
            &self.instances,
            &self.particles,
            &mut self.tex,
            self.camera.matrix(),
            context,
        );
        self.instances.clear();
        self.particles.clear();
        res
    }
}
