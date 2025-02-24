use crate::hittable::{HitRecord, Hittable};
use crate::ray::Ray;
 
#[derive(Default)]
pub struct HittableList {
    objects: Vec<Box<dyn Hittable>>,
}
 
impl HittableList {
    pub fn new() -> HittableList {
        Default::default()
    }
 
    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }

    pub fn objects(&self) -> &Vec<Box<dyn Hittable>> {
        &self.objects
    }
    
    // New mutable getter
    pub fn objects_mut(&mut self) -> &mut Vec<Box<dyn Hittable>> {
        &mut self.objects
    }
}
 
impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64, rec: &mut HitRecord) -> bool {
        let mut temp_rec = HitRecord::new();
        let mut hit_anything = false;
        let mut closest_so_far = t_max;
 
        for object in &self.objects {
            if object.hit(ray, t_min, closest_so_far, &mut temp_rec) {
                hit_anything = true;
                closest_so_far = temp_rec.t;
                *rec = temp_rec.clone();
            }
        }
 
        hit_anything
    }
}
