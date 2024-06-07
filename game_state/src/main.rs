//! An example of using the fBm noise function

extern crate noise;

use noise::{utils::*, Add, Cylinders, Fbm, MultiFractal, NoiseFn, Perlin, Worley};

fn main() {
    let fbm = Fbm::<Perlin>::new(12).set_frequency(0.1);
    let fbm_worley = Fbm::<Worley>::new(12).set_frequency(0.1);
    let noise = Add::new(fbm, fbm_worley);
    noise.get([0.2, 0.2, 0.2]);

    utils::write_example_to_file(
        &PlaneMapBuilder::new(Cylinders::new())
            .set_size(1000, 1000)
            .set_x_bounds(-5.0, 5.0)
            .set_y_bounds(-5.0, 5.0)
            .build(),
        "fbm_perlin.png",
    );
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
