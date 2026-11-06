use crate::color::Color;
use crate::pixel_color::PixelColor;

/// A sum of samples, normalized by the sample count when displayed.
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

    pub fn to_pixel_color(&self) -> PixelColor {
        let inv_samples = if self.samples > 0 {
            1.0 / (self.samples as f64)
        } else {
            0.0
        };
        PixelColor::new(
            self.r * inv_samples,
            self.g * inv_samples,
            self.b * inv_samples,
        )
    }
}

/// An already-normalized exponential moving average of frame colors.
///
/// Keeping this separate from `ColorAccumulator` prevents mixing summed samples
/// with an average or dividing an EMA by the number of frames a second time.
#[derive(Clone, Copy, Default)]
pub struct EmaColorAccumulator {
    color: Color,
    initialized: bool,
}

impl EmaColorAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Blend a frame's mean color using a weight in the range 0.0..=1.0.
    pub fn update(&mut self, new_sample: &Color, alpha: f64) {
        if self.initialized {
            self.color = alpha * *new_sample + (1.0 - alpha) * self.color;
        } else {
            // Start at the first sample instead of blending it with black.
            self.color = *new_sample;
            self.initialized = true;
        }
    }

    pub fn to_pixel_color(self) -> PixelColor {
        PixelColor::new(self.color.x(), self.color.y(), self.color.z())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba(pixel: PixelColor) -> (u8, u8, u8, u8) {
        (pixel.r, pixel.g, pixel.b, pixel.a)
    }

    #[test]
    fn empty_accumulators_display_opaque_black() {
        assert_eq!(
            rgba(ColorAccumulator::new().to_pixel_color()),
            (0, 0, 0, 255)
        );
        assert_eq!(
            rgba(EmaColorAccumulator::new().to_pixel_color()),
            (0, 0, 0, 255)
        );
        assert_eq!(
            rgba(EmaColorAccumulator::default().to_pixel_color()),
            (0, 0, 0, 255)
        );
    }

    #[test]
    fn summed_samples_are_averaged_once() {
        let mut accumulator = ColorAccumulator::new();
        accumulator.add_sample(&Color::new(1.0, 0.5, 0.0));
        accumulator.add_sample(&Color::new(0.0, 0.5, 1.0));
        accumulator.add_sample(&Color::new(0.5, 0.5, 0.5));
        assert_eq!(accumulator.samples, 3);
        assert_eq!(rgba(accumulator.to_pixel_color()), (127, 127, 127, 255));
    }

    #[test]
    fn first_ema_frame_is_not_blended_with_black() {
        let mut accumulator = EmaColorAccumulator::new();
        accumulator.update(&Color::new(1.0, 0.5, 0.25), 0.1);
        assert_eq!(rgba(accumulator.to_pixel_color()), (255, 127, 63, 255));
    }

    #[test]
    fn repeated_white_frames_keep_full_brightness() {
        let mut accumulator = EmaColorAccumulator::new();
        for frame in 1..=1000 {
            accumulator.update(&Color::new(1.0, 1.0, 1.0), 0.5);
            assert_eq!(
                rgba(accumulator.to_pixel_color()),
                (255, 255, 255, 255),
                "frame {frame}"
            );
        }
    }

    #[test]
    fn repeated_colored_frames_keep_each_channels_brightness() {
        let mut accumulator = EmaColorAccumulator::new();
        for frame in 1..=1000 {
            accumulator.update(&Color::new(0.25, 0.5, 0.75), 0.5);
            assert_eq!(
                rgba(accumulator.to_pixel_color()),
                (63, 127, 191, 255),
                "frame {frame}"
            );
        }
    }

    #[test]
    fn ema_weights_the_new_frame_and_retains_history() {
        let mut accumulator = EmaColorAccumulator::new();
        accumulator.update(&Color::new(1.0, 0.0, 0.0), 0.25);
        accumulator.update(&Color::new(0.0, 1.0, 0.0), 0.25);
        assert_eq!(rgba(accumulator.to_pixel_color()), (191, 63, 0, 255));
        accumulator.update(&Color::new(0.0, 0.0, 1.0), 0.25);
        assert_eq!(rgba(accumulator.to_pixel_color()), (143, 47, 63, 255));
    }

    #[test]
    fn zero_alpha_preserves_the_first_frame() {
        let mut accumulator = EmaColorAccumulator::new();
        accumulator.update(&Color::new(1.0, 0.5, 0.0), 0.0);
        accumulator.update(&Color::new(0.0, 0.0, 1.0), 0.0);
        assert_eq!(rgba(accumulator.to_pixel_color()), (255, 127, 0, 255));
    }

    #[test]
    fn unit_alpha_replaces_the_previous_frame() {
        let mut accumulator = EmaColorAccumulator::new();
        accumulator.update(&Color::new(1.0, 0.5, 0.0), 1.0);
        accumulator.update(&Color::new(0.0, 0.25, 1.0), 1.0);
        assert_eq!(rgba(accumulator.to_pixel_color()), (0, 63, 255, 255));
    }
}
