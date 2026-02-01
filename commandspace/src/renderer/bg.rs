use std::mem;

pub use crate::config::bg::BgConfig;
use crate::gl;
use crate::gl::types::*;
use crate::renderer;
use crate::renderer::shader::{ShaderError, ShaderProgram, ShaderVersion};

const RECT_SHADER_V: &str = include_str!("../../res/bg.v.glsl");
const BG_SHADER_F: &str = include_str!("../../res/bg.f.glsl");

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Vertex {
    x: f32,
    y: f32,
}

#[derive(Debug)]
pub struct BgRenderer {
    vao: GLuint,
    vbo: GLuint,
    program: BgShaderProgram,
    vertices: Vec<Vertex>,
}

impl BgRenderer {
    pub fn new(shader_version: ShaderVersion) -> Result<Self, renderer::Error> {
        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;

        let program = BgShaderProgram::new(shader_version)?;

        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // Position only
            gl::VertexAttribPointer(
                0, // location 0 in shader
                2, // 2 components: x, y
                gl::FLOAT,
                gl::FALSE,
                mem::size_of::<Vertex>() as i32,
                0 as *const _, // offset 0
            );
            gl::EnableVertexAttribArray(0);

            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }

        // Initialize vertices for a fullscreen quad
        let vertices = vec![
            Vertex { x: -1.0, y: -1.0 },
            Vertex { x: 3.0, y: -1.0 },
            Vertex { x: -1.0, y: 3.0 },
        ];

        Ok(Self { vao, vbo, program, vertices })
    }

    pub fn draw(&self, info: &BgConfig) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * mem::size_of::<Vertex>()) as isize,
                self.vertices.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );
        }

        unsafe {
            // Draw Bg
            let program = &self.program;
            gl::UseProgram(self.program.id());
            program.update_uniforms(info);
            gl::DrawArrays(gl::TRIANGLES, 0, self.vertices.len() as i32);

            // Disable program.
            gl::UseProgram(0);

            // Reset buffer bindings to nothing.
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
    }
}

impl Drop for BgRenderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

#[derive(Debug)]
pub struct BgShaderProgram {
    program: ShaderProgram,
    u_radius: Option<GLint>,
    u_bg_color: Option<GLint>,
    u_frame_color: Option<GLint>,
    u_frame_offset: Option<GLint>,
    u_frame_thickness: Option<GLint>,
}

impl BgShaderProgram {
    pub fn new(shader_version: ShaderVersion) -> Result<Self, ShaderError> {
        let program = ShaderProgram::new(shader_version, None, RECT_SHADER_V, BG_SHADER_F)?;

        Ok(Self {
            u_radius: program.get_uniform_location(c"radius").ok(),
            u_bg_color: program.get_uniform_location(c"bgColor").ok(),
            u_frame_color: program.get_uniform_location(c"frameColor").ok(),
            u_frame_offset: program.get_uniform_location(c"frameOffset").ok(),
            u_frame_thickness: program.get_uniform_location(c"frameThickness").ok(),
            program,
        })
    }

    fn id(&self) -> GLuint {
        self.program.id()
    }

    pub fn update_uniforms(&self, info: &BgConfig) {
        unsafe {
            if let Some(u_radius) = self.u_radius {
                gl::Uniform1f(u_radius, info.radius);
            }
            if let Some(u_bg_color) = self.u_bg_color {
                let (r, g, b) = info.bg_color.as_tuple();
                gl::Uniform4f(
                    u_bg_color,
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    info.bg_alpha,
                );
            }

            if info.frame_thickness == 0.0 {
                return;
            }
            if let Some(u_frame_color) = self.u_frame_color {
                let (r, g, b) = info.frame_color.as_tuple();
                gl::Uniform4f(
                    u_frame_color,
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    info.frame_alpha,
                );
            }
            if let Some(u_frame_offset) = self.u_frame_offset {
                gl::Uniform1f(u_frame_offset, info.frame_offset);
            }
            if let Some(u_frame_thickness) = self.u_frame_thickness {
                gl::Uniform1f(u_frame_thickness, info.frame_thickness);
            }
        }
    }
}
