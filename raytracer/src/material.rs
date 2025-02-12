use crate::color::Color;
use crate::hittable::HitRecord;
use crate::ray::Ray;
<<<<<<< HEAD

pub trait Material {
    fn scatter(
        &self, 
=======
use crate::vec3::{self, random_in_unit_sphere};

pub trait Material {
    fn scatter(
        &self,
>>>>>>> refs/remotes/origin/main
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool;
}
<<<<<<< HEAD
=======

pub struct Lambertian {
    albedo: Color,
}

impl Lambertian {
    pub fn new(a: Color) -> Lambertian {
        Lambertian { albedo: a}
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        _r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        let mut scatter_direction = rec.normal + vec3::random_in_unit_sphere();
        
        // If the random vector given is equal to the original ray they will create infinity/NaN
        // This deletes it
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        *attenuation = self.albedo;
        *scattered = Ray::new(rec.p, scatter_direction);
        true
    }
}


pub struct Metal {
    albedo: Color,
}

impl Metal {
    pub fn new(a: Color) -> Metal {
        Metal { albedo: a }
    }
}

impl Material for Metal {
    fn scatter (
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        let reflected = vec3::reflect(vec3::unit_vector(r_in.direction()), rec.normal);

        *attenuation = self.albedo;
        *scattered = Ray::new(rec.p, reflected);
        vec3::dot(scattered.direction(), rec.normal) > 0.0
    }
}
>>>>>>> refs/remotes/origin/main
