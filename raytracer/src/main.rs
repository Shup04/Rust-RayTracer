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
use color::Color;
use ray::Ray;
use vec3::{Point3, Vec3, random_in_unit_sphere};
use hittable::{HitRecord, Hittable};
use hittable_list::HittableList;

use camera::Camera;

use sphere::Sphere;
use cube::Cube;

fn hit_sphere(center: Point3, radius: f64, r: &Ray) -> f64 {
    let oc = r.origin() - center;
    let a = r.direction().length_squared();
    let half_b = vec3::dot(oc, r.direction());
    let c = oc.length_squared() - radius * radius;
    let discriminant = half_b * half_b - a * c;
    if discriminant < 0.0 {
        -1.0
    } else {
        (-half_b - f64::sqrt(discriminant)) / a
    }
}

fn ray_color(r: &Ray, world: &dyn Hittable, depth: i32) -> Color {
    // Skybox for rays that dont hit an object.
    let mut rec = HitRecord::new();

    if depth <= 0 {
        return Color::new(0.0, 0.0, 0.0); // Return black after max depth
    }

    if world.hit(r, 0.1, constants::INFINITY, &mut rec) {
        // Generate a random point in the unit sphere.
        let target = rec.p + rec.normal + random_in_unit_sphere();
        // Recursively trace the scattered ray.
        return 0.5 * ray_color(&Ray::new(rec.p, target - rec.p), world, depth - 1);
    }

    //let t = hit_sphere(Point3::new(0.0, 0.0, -1.0), 0.5, r);

    //if t > 0.0 {
    //    let n = vec3::unit_vector(r.at(t) - Vec3::new(0.0, 0.0, -1.0));
    //    return 0.5 * Color::new(n.x() + 1.0, n.y() + 1.0, n.z() + 1.0);
    //}

    //Skybox for rays that end at infinity.
    let unit_direction = vec3::unit_vector(r.direction());
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Color::new(0.81, 0.93, 0.96) + t * Color::new(0.28, 0.35, 0.50)
}

fn main() {
    //Image
    const ASPECT_RATIO: f64 = 16.0 / 9.0;
    const IMAGE_WIDTH: i32 = 1920;
    const IMAGE_HEIGHT: i32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as i32;
    const SAMPLES_PER_PIXEL: i32 = 1024;
    const MAX_DEPTH: i32 = 10;

    // World
    let mut world = HittableList::new();
    world.add(Box::new(Sphere::new(Point3::new(0.0, -1.0, -1.0), 0.5)));
    //world.add(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));
    world.add(Box::new(Cube::new(Point3::new(-2.0, -1.5, -2.0), Point3::new(-1.0, -0.5, -1.0))));
    world.add(Box::new(Cube::new(Point3::new(1.5, -0.75, -2.5), Point3::new(2.5, 0.25, -1.5))));

    world.add(Box::new(Cube::new(Point3::new(-5.0, -1.75, -2.5), Point3::new(5.0, -1.5, 1.5))));

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

                pixel_color += ray_color(&r, &world, MAX_DEPTH);
            }
            color::write_color(&mut io::stdout(), pixel_color, SAMPLES_PER_PIXEL);
        }
    }
    eprint!("Done");
}
