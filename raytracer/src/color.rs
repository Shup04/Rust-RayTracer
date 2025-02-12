 use std::io::Write;
 use crate::constants;
 use crate::vec3::Vec3;

// Type alias for color
pub type Color = Vec3;

pub fn write_color(out: &mut impl Write, pixel_color: Color, samples_per_pixel: i32) {
    // Take the R B G of a Vec3 and convert it to the 255 value and print
    let mut r = pixel_color.x();
    let mut g = pixel_color.y();
    let mut b = pixel_color.z();

    // We take multiple measurements for each pixel.
    // This is called a sample. So the color values could be way more than 100%.
    // Dividing by the number of samples gives us an average color of each sample,
    // thus reducing noise.
    let scale = 1.0 / samples_per_pixel as f64;
    r *= scale;
    g *= scale;
    b *= scale;

    writeln!(
        out,
        "{} {} {}",
        (256.0 * constants::clamp(r, 0.0, 0.999)) as i32, 
        (256.0 * constants::clamp(g, 0.0, 0.999)) as i32, 
        (256.0 * constants::clamp(b, 0.0, 0.999)) as i32, 
    ).expect("writing color");
}
