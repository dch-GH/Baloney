pub const GRAVITY: f32 = 9.82;

pub mod f32 {
    const PI: f32 = std::f32::consts::PI;

    pub fn radians_to_degrees(radians: f32) -> f32 {
        radians * (180.0 / PI)
    }

    pub fn degrees_to_radians(degrees: f32) -> f32 {
        degrees * (PI / 180.0)
    }
}

pub mod f64 {
    const PI: f64 = std::f64::consts::PI;

    pub fn radians_to_degrees(radians: f64) -> f64 {
        radians * (180.0 / PI)
    }

    pub fn degrees_to_radians(degrees: f64) -> f64 {
        degrees * (PI / 180.0)
    }
}

pub mod vector {
    pub mod random {
        use bevy::math::{Vec2, Vec3};
        use rand::prelude::*;

        pub fn vec2() -> Vec2 {
            let mut rng = rand::thread_rng();

            let ran_x: f32 = rng.gen_range(-1.0..1.0);
            let ran_y: f32 = rng.gen_range(-1.0..1.0);

            return bevy::math::vec2(ran_x, ran_y);
        }

        pub fn vec3() -> Vec3 {
            let mut rng = rand::thread_rng();

            let ran_x: f32 = rng.gen_range(-1.0..1.0);
            let ran_y: f32 = rng.gen_range(-1.0..1.0);
            let ran_z: f32 = rng.gen_range(-1.0..1.0);

            return bevy::math::vec3(ran_x, ran_y, ran_z);
        }
    }
}
