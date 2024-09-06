extern crate noise;

use noise::{utils::*, *};

fn main() {
    // Large slime bubble texture.
    let freq = 10.0;
    let red = Fbm::<Perlin>::new(0).set_frequency(freq);
    let green = Fbm::<Perlin>::new(1).set_frequency(freq);
    let blue = Fbm::<Perlin>::new(2).set_frequency(freq);

    // Finally, perturb the slime texture to add realism.
    // let final_slime = Turbulence::<_, Perlin>::new(large_slime)
    //     .set_seed(3)
    //     .set_frequency(8.0)
    //     .set_power(1.0 / 32.0)
    //     .set_roughness(2);

    let planar_red = PlaneMapBuilder::new(&red)
        .set_size(1024, 1024)
        .set_is_seamless(true)
        .build();
    let planar_green = PlaneMapBuilder::new(&green)
        .set_size(1024, 1024)
        .set_is_seamless(true)
        .build();
    let planar_blue = PlaneMapBuilder::new(&blue)
        .set_size(1024, 1024)
        .set_is_seamless(true)
        .build();

    // Create a slime palette.
    let red_grad = ColorGradient::new()
        .clear_gradient()
        .add_gradient_point(-1.0, [0, 0, 0, 255])
        .add_gradient_point(1.0, [255, 0, 0, 255]);

    let green_grad = ColorGradient::new()
        .clear_gradient()
        .add_gradient_point(-1.0, [0, 0, 0, 255])
        .add_gradient_point(1.0, [0, 255, 0, 255]);

    let blue_grad = ColorGradient::new()
        .clear_gradient()
        .add_gradient_point(-1.0, [0, 0, 0, 255])
        .add_gradient_point(1.0, [0, 0, 255, 255]);

    let red_image = ImageRenderer::new()
        .set_gradient(red_grad)
        .render(&planar_red);
    let green_image = ImageRenderer::new()
        .set_gradient(green_grad)
        .render(&planar_green);
    let blue_image = ImageRenderer::new()
        .set_gradient(blue_grad)
        .render(&planar_blue);

    let mut final_image = red_image;
    final_image
        .iter_mut()
        .zip(green_image.iter())
        .zip(blue_image.iter())
        .for_each(|((r, g), b)| {
            r[1] = g[1];
            r[2] = b[2];
            normalize_pixel(r);
        });

    utils::write_image_to_file(&final_image, "water_normals.png");
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
