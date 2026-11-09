use std::time::Instant;

mod camera;
mod color;
mod constants;
mod hittable;
mod hittable_list;
mod material;
mod pixel_color;
mod ray;
mod vec3;

mod cube;
mod sphere;
//mod volumetric;
mod color_accumulator;

use std::sync::Arc;

use color::Color;
use color_accumulator::EmaColorAccumulator;
use pixel_color::PixelColor;

use hittable::{HitRecord, Hittable};
use hittable_list::HittableList;
use ray::Ray;
use vec3::Point3;

use material::{Lambertian, Metal};

use camera::Camera;

use cube::Cube;
use sphere::Sphere;
//use volumetric::ConstantMedium;

use rayon::prelude::*;
use std::sync::Mutex;

//Rendering
const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: i32 = 1080;
const IMAGE_HEIGHT: i32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as i32;
const SAMPLES_PER_PIXEL: i32 = 5;
const MAX_DEPTH: i32 = 3;

const ALPHA: f64 = 0.5;

//Gravity
const DELTA_T: f64 = 0.1; // Time in between ray redirects caused by gravity.
const MAX_TIME: f64 = 10.0; // Total simulation time
const SINGULARITY: bool = false;

static RENDERER: Mutex<Option<Renderer>> = Mutex::new(None);

struct Renderer {
    width: usize,
    height: usize,
    samples: i32,
    seed: u64,
    pixels: Vec<PixelColor>,
    accum: Vec<EmaColorAccumulator>,
    world: HittableList,
    camera: Camera,
    started: Instant,
}

fn make_world() -> HittableList {
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
    world
}

impl Renderer {
    fn new(width: usize, height: usize, samples: i32, seed: u64) -> Self {
        Self {
            width,
            height,
            samples,
            seed,
            pixels: vec![PixelColor::new(0.0, 0.0, 0.0); width * height],
            accum: vec![EmaColorAccumulator::new(); width * height],
            world: make_world(),
            camera: Camera::with_aspect(width as f64 / height as f64),
            started: Instant::now(),
        }
    }

    fn render(&mut self, frame: i32) {
        let width = self.width;
        let height = self.height;
        let world = &self.world;
        let camera = &self.camera;
        let samples = self.samples;
        let seed = self.seed;
        self.accum
            .par_iter_mut()
            .zip(self.pixels.par_iter_mut())
            .enumerate()
            .for_each(|(index, (accum, pixel))| {
                // Seed each pixel/frame independently of Rayon scheduling.
                constants::seed_pixel(
                    seed ^ (index as u64).wrapping_mul(0x9e3779b97f4a7c15)
                        ^ (frame as u64).wrapping_mul(0xbf58476d1ce4e5b9),
                );
                let x = index % width;
                let y = height - 1 - index / width;
                let mut color = Color::default();
                for _ in 0..samples {
                    let u = (x as f64 + constants::random_double()) / (width - 1) as f64;
                    let v = (y as f64 + constants::random_double()) / (height - 1) as f64;
                    color += ray_color(
                        &camera.get_ray(u, v),
                        world,
                        MAX_DEPTH,
                        MAX_TIME,
                        DELTA_T,
                        SINGULARITY,
                    );
                }
                color = color / samples as f64;
                #[cfg(test)]
                assert!(color.x().is_finite() && color.y().is_finite() && color.z().is_finite());
                accum.update(&color, ALPHA);
                *pixel = accum.to_pixel_color();
            });
    }

    fn animate(&mut self, seconds: f64) {
        // Replace the known animated object without casting a trait-object pointer.
        // Absolute position keeps movement independent of the display frame rate.
        self.world.objects_mut()[0] = Box::new(Sphere::new(
            Point3::new(0.05 * seconds.sin(), -0.5, -1.0),
            0.5,
            Arc::new(Metal::new(Color::new(0.8, 0.8, 0.0), 1.0)),
        ));
    }
}

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
            if rec
                .mat
                .as_ref()
                .unwrap()
                .scatter(r, &rec, &mut attenuation, &mut scattered)
            {
                return attenuation
                    * ray_color(
                        &scattered,
                        world,
                        depth - 1,
                        max_t,
                        delta_t,
                        singularity_active,
                    );
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
            if rec
                .mat
                .as_ref()
                .unwrap()
                .scatter(&segment, &rec, &mut attenuation, &mut scattered)
            {
                return attenuation
                    * ray_color(
                        &scattered,
                        world,
                        depth - 1,
                        max_t,
                        delta_t,
                        singularity_active,
                    );
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

// Single-caller C ABI. The pointer is borrowed until reinitialization/shutdown;
// callers must not read it concurrently with update_image.
#[no_mangle]
pub extern "C" fn initialize_image_with_options(
    width: i32,
    height: i32,
    samples: i32,
    seed: u64,
) -> i32 {
    if width < 2
        || height < 2
        || width > 4096
        || height > 4096
        || i64::from(width) * i64::from(height) > 4_194_304
        || !(1..=1024).contains(&samples)
    {
        return 0;
    }
    *RENDERER.lock().unwrap() = Some(Renderer::new(
        width as usize,
        height as usize,
        samples,
        seed,
    ));
    1
}

#[no_mangle]
pub extern "C" fn initialize_image() {
    initialize_image_with_options(IMAGE_WIDTH, IMAGE_HEIGHT, SAMPLES_PER_PIXEL, 1);
}

#[no_mangle]
pub extern "C" fn update_image(frame: i32) {
    if let Some(renderer) = RENDERER.lock().unwrap().as_mut() {
        renderer.render(frame);
    }
}

#[no_mangle]
pub extern "C" fn animate_sphere_simple() {
    if let Some(renderer) = RENDERER.lock().unwrap().as_mut() {
        renderer.animate(renderer.started.elapsed().as_secs_f64());
    }
}

#[no_mangle]
pub extern "C" fn animate_sphere_at(seconds: f64) {
    if seconds.is_finite() {
        if let Some(renderer) = RENDERER.lock().unwrap().as_mut() {
            renderer.animate(seconds);
        }
    }
}

#[no_mangle]
pub extern "C" fn get_image_ptr() -> *const PixelColor {
    RENDERER
        .lock()
        .unwrap()
        .as_ref()
        .map_or(std::ptr::null(), |r| r.pixels.as_ptr())
}

#[no_mangle]
pub extern "C" fn get_image_width() -> i32 {
    RENDERER
        .lock()
        .unwrap()
        .as_ref()
        .map_or(0, |r| r.width as i32)
}

#[no_mangle]
pub extern "C" fn get_image_height() -> i32 {
    RENDERER
        .lock()
        .unwrap()
        .as_ref()
        .map_or(0, |r| r.height as i32)
}

#[no_mangle]
pub extern "C" fn shutdown_image() {
    *RENDERER.lock().unwrap() = None;
}

#[cfg(test)]
mod renderer_tests {
    use super::*;

    fn bytes(renderer: &Renderer) -> Vec<(u8, u8, u8, u8)> {
        renderer
            .pixels
            .iter()
            .map(|p| (p.r, p.g, p.b, p.a))
            .collect()
    }

    #[test]
    fn deterministic_finite_render_across_worker_counts() {
        let render = |workers| {
            rayon::ThreadPoolBuilder::new()
                .num_threads(workers)
                .build()
                .unwrap()
                .install(|| {
                    let mut r = Renderer::new(32, 18, 2, 42);
                    for frame in 0..12 {
                        r.render(frame);
                    }
                    assert!(r.pixels.iter().all(|p| p.a == 255));
                    assert!(r.pixels.iter().any(|p| p.r > 0));
                    bytes(&r)
                })
        };
        assert_eq!(render(1), render(4));
    }

    #[test]
    fn static_render_keeps_brightness_over_many_frames() {
        let mut r = Renderer::new(32, 18, 2, 123);
        let mut means = Vec::new();
        for frame in 0..100 {
            r.render(frame);
            means.push(
                r.pixels
                    .iter()
                    .map(|p| f64::from(p.r) + f64::from(p.g) + f64::from(p.b))
                    .sum::<f64>()
                    / (32.0 * 18.0 * 3.0),
            );
        }
        let early = means[5..15].iter().sum::<f64>() / 10.0;
        let late = means[90..100].iter().sum::<f64>() / 10.0;
        assert!(
            (late / early - 1.0).abs() < 0.1,
            "brightness early={early}, late={late}"
        );
    }

    #[test]
    fn absolute_animation_is_repeatable_after_reinitialization() {
        let mut a = Renderer::new(32, 18, 1, 5);
        a.animate(100.0);
        a.animate(1.0);
        a.render(0);
        let mut b = Renderer::new(32, 18, 1, 5);
        b.animate(1.0);
        b.render(0);
        assert_eq!(bytes(&a), bytes(&b));
    }
}
