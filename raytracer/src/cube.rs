use crate::hittable::{HitRecord, Hittable};
use crate::ray::Ray;
use crate::vec3::{self, Point3, Vec3};

pub struct Cube {
    a: Point3, // Min Corner
    b: Point3, // Max Corner
}

impl Cube {
    pub fn new(min: Point3, max: Point3) -> Cube {
        Cube {
            a: min,
            b: max,
        }
    }
}

impl Hittable for Cube {
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        // Compute t values for x range.
        let t_x0 = (self.a.x() - r.origin().x()) / r.direction().x();
        let t_x1 = (self.b.x() - r.origin().x()) / r.direction().x();
        let t_xMin = t_x0.min(t_x1); // X range entry
        let t_xMax = t_x0.max(t_x1); // X range exit

        // Compute t valin this case, im assuming only positive directions, not rendering anything behind the camera.ues for y range.
        let t_y0 = (self.a.y() - r.origin().y()) / r.direction().y();
        let t_y1 = (self.b.y() - r.origin().y()) / r.direction().y();
        let t_yMin = t_y0.min(t_y1); // Y range entry
        let t_yMax = t_y0.max(t_y1); // Y range exit

        // Compute t values for z range.
        let t_z0 = (self.a.z() - r.origin().z()) / r.direction().z();
        let t_z1 = (self.b.z() - r.origin().z()) / r.direction().z();
        let t_zMin = t_z0.min(t_z1); // Z range entry
        let t_zMax = t_z0.max(t_z1); // Z range exit

        // Calculate t entry & exit of each Ray 
        // Since we know the entry and exit of the x, y, z ranges,
        // the entry is the last range thew ray enters.
        // In other words, the max of the entries.
        let t_entry = t_xMin.max(t_yMin).max(t_zMin);
        let t_exit = t_xMax.min(t_yMax).min(t_zMax);

        if t_entry <= t_exit && t_entry < t_max && t_exit > t_min {
            rec.t = t_entry;
            rec.p = r.at(t_entry);

        // Determine which face was hit:
        let epsilon = 1e-6;
        if (t_entry - t_xMin).abs() < epsilon {
            // Hit on an x-face.
            if r.direction().x() > 0.0 {
                rec.normal = Vec3::new(-1.0, 0.0, 0.0);
            } else {
                rec.normal = Vec3::new(1.0, 0.0, 0.0);
            }
        } else if (t_entry - t_yMin).abs() < epsilon {
            // Hit on a y-face.
            if r.direction().y() > 0.0 {
                rec.normal = Vec3::new(0.0, -1.0, 0.0);
            } else {
                rec.normal = Vec3::new(0.0, 1.0, 0.0);
            }
        } else if (t_entry - t_zMin).abs() < epsilon {
            // Hit on a z-face.
            if r.direction().z() > 0.0 {
                rec.normal = Vec3::new(0.0, 0.0, -1.0);
            } else {
                rec.normal = Vec3::new(0.0, 0.0, 1.0);
            }
        }
        // Optionally, use rec.set_face_normal(r, rec.normal) if you want to
        // ensure the normal always points outward relative to the ray.

        return true;
        }
        false
    }
}
