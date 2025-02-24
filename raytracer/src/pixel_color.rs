#[repr(C)]
pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl PixelColor {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self {
            r: (r * 255.0).clamp(0.0, 255.0) as u8,
            g: (g * 255.0).clamp(0.0, 255.0) as u8,
            b: (b * 255.0).clamp(0.0, 255.0) as u8,
            a: 255,
        }
    }
}
