use std::time::{Instant, Duration};

mod vec3;
mod color;
mod pixel_color;
mod ray;
mod constants;
mod hittable;
mod hittable_list;
mod camera;
mod material;

mod sphere;
mod cube;
//mod volumetric;

use std::io;
use std::sync::Arc;

use color::Color;
use pixel_color::PixelColor;
use ray::Ray;
use vec3::{Point3, Vec3, random_in_unit_sphere};
use hittable::{HitRecord, Hittable};
use hittable_list::HittableList;
use rand::Rng;

use material::{Lambertian, Metal};

use camera::Camera;

use sphere::Sphere;
use cube::Cube;
//use volumetric::ConstantMedium;

use rayon::prelude::*;
use std::sync::Mutex;
#[macro_use]
extern crate lazy_static;

//Rendering
const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: i32 = 1080;
const IMAGE_HEIGHT: i32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as i32;
const SAMPLES_PER_PIXEL: i32 = 1;
const MAX_DEPTH: i32 = 3;

//Gravity
const DELTA_T: f64 = 0.1; // Time in between ray redirects caused by gravity.
const MAX_TIME: f64 = 10.0; // Total simulation time
const SINGULARITY: bool = false;

static mut IMAGE_BUFFER: Option<Vec<PixelColor>> = None;

// World and Camera Mutexes
lazy_static! {
    static ref START_TIME: Instant = Instant::now();

    static ref WORLD: Mutex<HittableList> = Mutex::new({
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
    });

    static ref CAM: Mutex<Camera> = Mutex::new(Camera::new());
}

static mut IMAGE: Option<Vec<PixelColor>> = None;


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


pub fn compute_image(buffer: &mut [PixelColor], frame: i32) {
    // Clone shared state for thread safety.
    let cam = CAM.lock().unwrap();
    let world = WORLD.lock().unwrap();
    let width_usize = IMAGE_WIDTH as usize;

    buffer.par_iter_mut().enumerate().for_each(|(index, pixel)| {
        let x = index % width_usize;
        let y = IMAGE_HEIGHT - 1 - (index / width_usize) as i32;

        let grid_spacing = 3;
        let total = grid_spacing * grid_spacing; // 25
        let idx = frame % total;
        let offset_x = idx % grid_spacing;
        let offset_y = idx / grid_spacing; // integer division

        if (x as i32 % grid_spacing == offset_x) && (y % grid_spacing == offset_y) {
            let mut accum_color = Color::new(0.0, 0.0, 0.0);
            for _ in 0..SAMPLES_PER_PIXEL {
                let u = (x as f64 + constants::random_double() * 1.0) / ((IMAGE_WIDTH - 1) as f64);
                let v = (y as f64 + constants::random_double() * 1.0) / ((IMAGE_HEIGHT - 1) as f64);
                let r = cam.get_ray(u, v);
                accum_color += ray_color(&r, &*world, MAX_DEPTH, MAX_TIME, DELTA_T, SINGULARITY);
            }
            // Average the color samples.
            *pixel = PixelColor::new(
                accum_color.x() / (SAMPLES_PER_PIXEL as f64),
                accum_color.y() / (SAMPLES_PER_PIXEL as f64),
                accum_color.z() / (SAMPLES_PER_PIXEL as f64),
            );
        }
    });
}

#[no_mangle]
pub extern "C" fn animate_sphere_simple() {
    // Lock the world.
    let mut world = WORLD.lock().unwrap();
    // Assume the sphere to animate is at index 0.
    if let Some(obj) = world.objects_mut().get_mut(0) {
        // UNSAFELY assume the object is a Sphere.
        let sphere = unsafe { &mut *(obj.as_mut() as *mut dyn Hittable as *mut Sphere) };
        // Animate the sphere’s x position with a simple sine over time.
        let time = START_TIME.elapsed().as_secs_f64();
        let amplitude = 0.05;
        sphere.set_center(Point3::new(sphere.center().x() + amplitude * time.sin(), sphere.center().y(), sphere.center().z()));
    }
}

// External functions for the cpp code to call for rendering.
#[no_mangle]
pub extern "C" fn initialize_image() {
    unsafe {
        // Allocate a vector with the desired number of pixels.
        IMAGE_BUFFER = Some(vec![PixelColor::default(); (IMAGE_WIDTH * IMAGE_HEIGHT) as usize]);
    }
}

#[no_mangle]
pub extern "C" fn update_image(frame: i32) {
    unsafe {
        if let Some(ref mut buffer) = IMAGE_BUFFER {
            compute_image(buffer, frame);
        }
    }
}

#[no_mangle]
pub extern "C" fn get_image_ptr() -> *const PixelColor {
    unsafe {
        IMAGE_BUFFER
            .as_ref()
            .map_or(std::ptr::null(), |buffer| buffer.as_ptr())
    }
}

#[no_mangle]
pub extern "C" fn get_image_width() -> i32 {
    IMAGE_WIDTH as i32
}

#[no_mangle]
pub extern "C" fn get_image_height() -> i32 {
    IMAGE_HEIGHT as i32
}

fn main(){}
