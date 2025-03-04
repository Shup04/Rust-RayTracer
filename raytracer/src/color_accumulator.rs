use crate::color::Color;
use crate::pixel_color::PixelColor;

#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct ColorAccumulator {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub samples: i32,
}

impl ColorAccumulator {
    pub fn new() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            samples: 0,
        }
    }

    pub fn add_sample(&mut self, sample: &Color) {
        self.r += sample.x();
        self.g += sample.y();
        self.b += sample.z();
        self.samples += 1;
    }

    pub fn update_ema(&mut self, new_sample: &Color, alpha: f64) {
        if self.samples == 0 {
            // First sample: initialize the accumulator.
            self.r = new_sample.x();
            self.g = new_sample.y();
            self.b = new_sample.z();
        } else {
            self.r = alpha * new_sample.x() + (1.0 - alpha) * self.r;
            self.g = alpha * new_sample.y() + (1.0 - alpha) * self.g;
            self.b = alpha * new_sample.z() + (1.0 - alpha) * self.b;
        }
        self.samples += 1;
    }

    pub fn to_pixel_color(&self) -> PixelColor {
        let inv_samples = if self.samples > 0 {
            1.0 / (self.samples as f64)
        } else {
            0.0
        };
        PixelColor::new(self.r * inv_samples, self.g * inv_samples, self.b * inv_samples)
    }
}
