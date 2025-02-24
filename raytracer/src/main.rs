use std::time::{Instant, Duration};

mod vec3;
mod color;
mod ray;
mod constants;
mod hittable;
mod hittable_list;
mod camera;
mod material;

mod sphere;
mod cube;
mod volumetric;

use std::io;
use std::sync::Arc;

use color::Color;
use ray::Ray;
use vec3::{Point3, Vec3, random_in_unit_sphere};
use hittable::{HitRecord, Hittable};
use hittable_list::HittableList;
use rand::Rng;

use material::{Lambertian, Metal};

use camera::Camera;

use sphere::Sphere;
use cube::Cube;
use volumetric::ConstantMedium;

use rayon::prelude::*;

fn ray_color(
    r: &Ray,
    world: &dyn Hittable,
    depth: i32,
    max_t: f64,
    delta_t: f64,
    singularity_active: bool,
) -> Color {
    if depth <= 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    let mut rec = HitRecord::new();

    // Case 1: No integrated ray intersections (standard ray tracing)
    if !singularity_active {
        if world.hit(r, 0.001, max_t, &mut rec) {
            let mut attenuation = Color::default();
            let mut scattered = Ray::default();
            if rec.mat.as_ref().unwrap().scatter(r, &rec, &mut attenuation, &mut scattered) {
                return attenuation * ray_color(&scattered, world, depth - 1, max_t, delta_t, singularity_active);
            }
            return Color::new(0.0, 0.0, 0.0);
        }
        // Return sky color if no hit
        let unit_direction = vec3::unit_vector(r.direction());
        let t = 0.5 * (unit_direction.y() + 1.0);
        return (1.0 - t) * Color::new(0.81, 0.93, 0.96) + t * Color::new(0.28, 0.35, 0.50);
    }

    // Case 2: Integrated ray intersections with gravitational bending.
    // Gravitational parameters.
    const G: f64 = 6.6743e-11;
    let singularity = Point3::new(0.0, -0.5, -1.0);
    let mass: f64 = 3.5e9;
    const SEGMENT_LENGTH: f64 = 0.1; 

    let mut pos = r.origin();
    let mut dir = r.direction().normalize();
    let mut t_total = 0.0;
    
    while t_total <= max_t {
        let segment = Ray::new(pos, dir);
        // Check if any object is hit within the next SEGMENT_LENGTH.
        if world.hit(&segment, 0.001, SEGMENT_LENGTH, &mut rec) {
            let mut attenuation = Color::default();
            let mut scattered = Ray::default();
            if rec.mat.as_ref().unwrap().scatter(&segment, &rec, &mut attenuation, &mut scattered) {
                return attenuation * ray_color(&scattered, world, depth - 1, max_t, delta_t, singularity_active);
            }
            return Color::new(0.0, 0.0, 0.0);
        }
        
        // Update gravitational acceleration.
        let r_vec = pos - singularity;
        let R = r_vec.length();
        if R < 1e-6 {
            break;
        }
        let r_hat = r_vec / R;
        let a = -G * mass / (R * R) * r_hat;

        // Update the direction and position.
        dir = (dir + a * delta_t).normalize();
        pos = pos + dir * delta_t;
        t_total += delta_t;
    }
    
    // Return sky color if no hit was detected.
    let unit_direction = vec3::unit_vector(dir);
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Color::new(0.81, 0.93, 0.96) + t * Color::new(0.28, 0.35, 0.50)
}

fn compute_pixel(x: i32, y: i32) -> Color {
    let mut pixel: Color = Color::new(0.0, 0.0, 0.0);
    for _ in 0..SAMPLES_PER_PIXEL {
        let u = (x as f64 + constants::random_double()) / (IMAGE_WIDTH - 1) as f64;
        let v = (y as f64 + constants::random_double()) / (IMAGE_HEIGHT - 1) as f64;
        let r = cam.get_ray(u, v);
        pixel_color += ray_color(&r, &world, MAX_DEPTH, MAX_TIME, DELTA_T, SINGULARITY);
    }
}

fn main() {
    //Image
    const ASPECT_RATIO: f64 = 16.0 / 9.0;
    const IMAGE_WIDTH: i32 = 1080;
    const IMAGE_HEIGHT: i32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as i32;
    const SAMPLES_PER_PIXEL: i32 = 128;
    const MAX_DEPTH: i32 = 15;

    //Gravity
    const DELTA_T: f64 = 0.1; // Time in between ray redirects caused by gravity.
    const MAX_TIME: f64 = 10.0; // Total simulation time
    const SINGULARITY: bool = false;

    // World
    let mut world = HittableList::new();

    let ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let sphere1 = Arc::new(Metal::new(Color::new(0.8, 0.8, 0.0), 1.0));
    let sphere2 = Arc::new(Metal::new(Color::new(0.8, 0.8, 0.6), 1.0));
    let left_cube = Arc::new(Metal::new(Color::new(1.0, 0.4, 0.8), 0.3));
    let right_cube = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));

    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -0.5, -1.0), 
        0.5,
        sphere1,
    )));

    world.add(Box::new(Sphere::new(
        Point3::new(3.0, -0.5, -1.0), 
        1.0,
        sphere2,
    )));

    world.add(Box::new(Cube::new(
        Point3::new(-2.0, -1.5, 2.0), 
        Point3::new(-1.0, 1.0, -3.0),
        left_cube,
    )));
    world.add(Box::new(Cube::new(
        Point3::new(0.5, -0.75, -2.5), 
        Point3::new(1.5, 0.25, -1.5),
        right_cube,
    )));

    world.add(Box::new(Cube::new(
        Point3::new(-5.0, -1.75, -5.5), 
        Point3::new(5.0, -1.5, 1.5),
        ground,
    )));

    // Camera
    let cam = Camera::new();

    //Render
    //
    let mut image: Vec<Color> = Vec::with_capacity((IMAGE_WIDTH * IMAGE_HEIGHT) as usize) ;
    for y in 0..IMAGE_HEIGHT {
        for x in 0..IMAGE_WIDTH {
            let mut pixel = Color::new(0.0, 0.0, 0.0);
            for _ in 0..SAMPLES_PER_PIXEL {
                let u = (x as f64 + constants::random_double()) / (IMAGE_WIDTH - 1) as f64;
                let v = (y as f64 + constants::random_double()) / (IMAGE_HEIGHT - 1) as f64;
                let r = cam.get_ray(u, v);
                pixel += ray_color(&r, &world, MAX_DEPTH, MAX_TIME, DELTA_T, SINGULARITY);
            }
            image.push(pixel);
        }
    }

    for j in (0..IMAGE_HEIGHT).rev() {
        scanlines_done = j;
        let scanline_start = Instant::now();

        let row: Vec<Color> = (0..IMAGE_WIDTH)
            .into_par_iter()
            .map(|i| {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                for _ in 0..SAMPLES_PER_PIXEL {
                    let u = (i as f64 + constants::random_double()) / (IMAGE_WIDTH - 1) as f64;
                    let v = (j as f64 + constants::random_double()) / (IMAGE_HEIGHT - 1) as f64;
                    let r = cam.get_ray(u, v);
                    pixel_color += ray_color(&r, &world, MAX_DEPTH, MAX_TIME, DELTA_T, SINGULARITY);
                }
                pixel_color
            })
            .collect();

    // Write the computed scanline to stdout in order.
    for pixel in row {
        color::write_color(&mut io::stdout(), pixel, SAMPLES_PER_PIXEL);
    }
        // End timing for this scanline and update our running total.
        let scanline_duration = scanline_start.elapsed();
        total_scanline_time += scanline_duration;
        scanlines_done += 1;
        
        // Calculate average time per scanline so far.
        let avg_time = total_scanline_time / scanlines_done as u32;
        let scanlines_remaining = j; // since j counts down
        let estimated_remaining = avg_time * scanlines_remaining as u32; // multiplication works with Duration
        
        eprint!(
            "Scanlines remaining: {}. Estimated time remaining: {:?}\r",
            j, estimated_remaining
        );
    }
    eprint!("Done");
}
