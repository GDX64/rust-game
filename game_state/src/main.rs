extern crate noise;

use noise::{utils::*, *};

const SIZE: usize = 1024;

fn make_noise_image(seed: u32) -> NoiseImage {
    // Large slime bubble texture.
    let freq = 0.003;
    let red = Fbm::<Perlin>::new(seed).set_frequency(freq).set_octaves(8);
    let z_gain = 50.0;

    let domain_x = Fbm::<Perlin>::new(100).set_frequency(freq);
    let domain_y = Fbm::<Perlin>::new(101).set_frequency(freq);

    let warped_noise = |point: [f64; 2]| {
        // let x = domain_x.get(point);
        // let y = domain_y.get(point);
        // return red.get([x, y]);
        return red.get(point);
    };

    let mut grid = vec![[0.0; SIZE]; SIZE];

    for y in 0..SIZE {
        for x in 0..SIZE {
            let point = [x as f64, y as f64];
            grid[y][x] = warped_noise(point) * z_gain;
        }
    }

    let mut image = NoiseImage::new(SIZE, SIZE);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let z = grid[y][x];
            let prev_x = if x == 0 { 0 } else { x - 1 };
            let prev_zx = grid[y][prev_x];
            let dz_dx = z - prev_zx;
            let prev_y = if y == 0 { 0 } else { y - 1 };
            let prev_zy = grid[prev_y][x];
            let dz_dy = z - prev_zy;

            let grad_x = Vector3::new(1.0, 0.0, dz_dx);
            let grad_y = Vector3::new(0.0, 1.0, dz_dy);
            let normal = grad_x.cross(grad_y).normalize();
            let normal = (normal + 1.0).normalize() * 255.0;
            image.set_value(x, y, [normal.x as u8, normal.y as u8, normal.z as u8, 255]);
        }
    }

    image
}

fn main() {
    let image = make_noise_image(0);

    utils::write_image_to_file(&image, "water_normals.png");
}
mod utils {
    use noise::utils::{NoiseImage, NoiseMap};
    pub fn write_example_to_file(map: &NoiseMap, filename: &str) {
        use std::{fs, path::Path};

        let target = Path::new("example_images/").join(Path::new(filename));

        fs::create_dir_all(target.clone().parent().expect("No parent directory found."))
            .expect("Failed to create directories.");

        map.write_to_file(&target)
    }

    pub fn write_image_to_file(image: &NoiseImage, filename: &str) {
        use std::{fs, path::Path};

        let target = Path::new("example_images/").join(Path::new(filename));

        fs::create_dir_all(target.clone().parent().expect("No parent directory found."))
            .expect("Failed to create directories.");

        image.write_to_file(&target)
    }
}

fn normalize_pixel(p: &mut [u8; 4]) {
    let r = p[0] as f32 / 255.0;
    let g = p[1] as f32 / 255.0;
    let b = p[2] as f32 / 255.0;
    let len = (r * r + g * g + b * b).sqrt();
    let r = r / len * 255.0;
    let g = g / len * 255.0;
    let b = b / len * 255.0;
    p[0] = r as u8;
    p[1] = g as u8;
    p[2] = b as u8;
}
