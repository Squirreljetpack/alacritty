use super::rgb::Rgb;

#[derive(Debug, Copy, Clone)]
pub struct BgConfig {
    pub radius: f32,
    pub bg_color: Rgb,
    pub bg_alpha: f32,
    pub frame_color: Rgb,
    pub frame_alpha: f32,
    pub frame_offset: f32,
    pub frame_thickness: f32,
}
