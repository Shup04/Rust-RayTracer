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

use std::io;
use std::rc::Rc;

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

fn integrate_ray_path(r: &Ray, max_t: f64, delta_t: f64) -> Ray {
    const G: f64 = 6.6743e-11; // gravitational constant
    let singularity = Point3::new(0.0, -0.5, -1.0);
    // Try using an exaggerated mass for visual effect.
    let mass: f64 = 3.5e9; // Adjust this value as needed
    let mut t = 0.0;

    // Start with the ray's current origin and direction.
    let mut pos = r.origin();
    let mut dir = r.direction().normalize();

    while t < max_t {
        // Calculate the vector from the singularity to the current position.
        let r_vec = pos - singularity;
        let R = r_vec.length();
        if R < 1e-6 {
            break;
        }
        
        // Compute the unit vector from the singularity to pos.
        let r_hat = r_vec / R;
        
        // Compute gravitational acceleration: a = -G * mass / R^2 * r_hat
        let a = -G * mass / (R * R) * r_hat;
        
        // Update the direction: add a * delta_t and normalize.
        dir = (dir + a * delta_t).normalize();
        
        // Update the position: move along the new direction.
        pos = pos + dir * delta_t;
        
        t += delta_t;
    }

    // Return the new ray with the updated position and direction.
    Ray::new(pos, dir)
}

fn ray_color(
    r: &Ray,
    world: &dyn Hittable,
    depth: i32,
    max_t: f64,
    delta_t: f64
) -> Color {
    if depth <= 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    // Gravitational parameters.
    const G: f64 = 6.6743e-11;
    let singularity = Point3::new(0.0, -0.5, -1.0);
    let mass: f64 = 3.5e9;

    // Use a segment length that is better matched to your scene scale.
    const SEGMENT_LENGTH: f64 = 0.1; // For example, 0.1 units

    let mut pos = r.origin();
    let mut dir = r.direction().normalize();
    let mut t_total = 0.0;
    let mut rec = HitRecord::new();

    while t_total < max_t {
        let segment = Ray::new(pos, dir);

        // Check if any object is hit within the next SEGMENT_LENGTH.
        if world.hit(&segment, 0.001, SEGMENT_LENGTH, &mut rec) {
            let mut attenuation = Color::default();
            let mut scattered = Ray::default();
            if rec.mat.as_ref().unwrap().scatter(&segment, &rec, &mut attenuation, &mut scattered) {
                return attenuation * ray_color(&scattered, world, depth - 1, max_t, delta_t);
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

fn main() {
    //Image
    const ASPECT_RATIO: f64 = 16.0 / 9.0;
    const IMAGE_WIDTH: i32 = 256;
    const IMAGE_HEIGHT: i32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as i32;
    const SAMPLES_PER_PIXEL: i32 = 50;
    const MAX_DEPTH: i32 = 15;

    //Gravity
    const DELTA_T: f64 = 0.1; // Time in between ray redirects caused by gravity.
    const MAX_TIME: f64 = 10.0; // Total simulation time

    // World
    let mut world = HittableList::new();

    let ground = Rc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let sphere1 = Rc::new(Metal::new(Color::new(0.8, 0.8, 0.0), 1.0));
    let left_cube = Rc::new(Metal::new(Color::new(1.0, 0.4, 0.8), 0.0));
    let right_cube = Rc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));

    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -0.5, -1.0), 
        0.1,
        sphere1,
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
    print!("P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT);

    for j in (0..IMAGE_HEIGHT).rev() {
        eprint!("Scanlines remaining: {}", j);
        for i in 0..IMAGE_WIDTH {
            let mut pixel_color = Color::new(0.0, 0.0, 0.0);
            for _ in 0..SAMPLES_PER_PIXEL {
                let u = (i as f64 + constants::random_double()) / (IMAGE_WIDTH - 1) as f64;
                let v = (j as f64 + constants::random_double()) / (IMAGE_HEIGHT - 1) as f64;
                let r = cam.get_ray(u, v);

                pixel_color += ray_color(&r, &world, MAX_DEPTH, MAX_TIME, DELTA_T);
            }
            color::write_color(&mut io::stdout(), pixel_color, SAMPLES_PER_PIXEL);
        }
    }
    eprint!("Done");
}
