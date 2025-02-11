 use std::io::write;

 use crate::vec3::Vec3;

// Type alias for color
pub type Color = Vec3;

pub fn write_color(out: &mut impl Write, pixel_color: Color) {
    // Take the R B G of a Vec3 and convert it to the 255 value and print
    let r = (255.999, pixel_color.x()) as i32;
    let g = (255.999, pixel_color.y()) as i32;
    let b = (255.999, pixel_color.y()) as i32;
    writeln!(out, "{} {} {}", r, g, b).expect("writing color");
}
