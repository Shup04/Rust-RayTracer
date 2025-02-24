#[derive(Clone, Debug)]
pub struct AABB {
    pub minimum: Vec3,
    pub maximum: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { minimum: min, maximum: max }
    }

    // A helper function that returns the surrounding box.
    pub fn surrounding_box(box0: &AABB, box1: &AABB) -> AABB {
        let small = Vec3::new(
            box0.minimum.x().min(box1.minimum.x()),
            box0.minimum.y().min(box1.minimum.y()),
            box0.minimum.z().min(box1.minimum.z()),
        );
        let big = Vec3::new(
            box0.maximum.x().max(box1.maximum.x()),
            box0.maximum.y().max(box1.maximum.y()),
            box0.maximum.z().max(box1.maximum.z()),
        );
        AABB::new(small, big)
    }

    // Intersection method for a ray.
    pub fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> bool {
        // ... implement ray-box intersection ...
        
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
            return true;
        }
        false
    }
}
