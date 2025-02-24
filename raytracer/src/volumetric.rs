use std::sync::Arc;
use crate::{Color, HitRecord, Hittable, Ray, AABB};
use crate::material::Isotropic;
use crate::material::Material;
use crate::vec3::Vec3;
use rand::random;

pub struct ConstantMedium {
    boundary: Arc<dyn Hittable>,
    neg_inv_density: f64,
    phase_function: Arc<dyn Material>,
}

impl ConstantMedium {
    pub fn new(
        boundary: Arc<dyn Hittable>,
        density: f64,
        phase_function: Arc<dyn Material>,
    ) -> self {
        self {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function,
        }
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        // Get the first intersection with the boundary.
        let mut rec1 = HitRecord::new();
        let mut rec2 = HitRecord::new();

        if !self.boundary.hit(r, -f64::INFINITY, f64::INFINITY, &mut rec1) {
            return false;
        }
        if !self.boundary.hit(r, rec1.t + 0.0001, f64::INFINITY, &mut rec2) {
            return false;
        }

        // Clamp the intersection distances.
        let t_enter = rec1.t.max(t_min);
        let t_exit = rec2.t.min(t_max);
        if t_enter >= t_exit {
            return false;
        }

        // Ensure t_enter is non-negative.
        let t_enter = if t_enter < 0.0 { 0.0 } else { t_enter };

        // Compute the distance the ray travels inside the medium.
        let ray_length = r.direction().length();
        let distance_inside_boundary = (t_exit - t_enter) * ray_length;

        // Sample a distance along the ray.
        let hit_distance = self.neg_inv_density * random::<f64>().ln();

        if hit_distance > distance_inside_boundary {
            // No scattering event within the medium.
            return false;
        }

        // Record the scattering hit at t_hit along the ray.
        rec.t = t_enter + hit_distance / ray_length;
        rec.p = r.at(rec.t);
        rec.normal = Vec3::new(1.0, 0.0, 0.0); // arbitrary; not used in isotropic scattering.
        rec.mat = Some(self.phase_function.clone());

        true
    }

    fn bounding_box(&self, time0: f64, time1: f64) -> Option<AABB> {
        self.boundary.bounding_box(time0, time1)
    }
}
