use std::ops::AddAssign;
use crate::color::Color;

#[derive(Clone, Copy, Default)]
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

    pub fn x(&self) -> u8 {
        self.r
    }
 
    pub fn y(&self) -> u8 {
        self.g
    }

    pub fn z(&self) -> u8 {
        self.b
    }
}

impl AddAssign for PixelColor {
    fn add_assign(&mut self, other: Self) {
        self.r = self.r.saturating_add(other.r);
        self.g = self.g.saturating_add(other.g);
        self.b = self.b.saturating_add(other.b);
        self.a = self.a.saturating_add(other.a); // useless for now since alpha is always 100%
    }
}

