extern crate camera_capture;

use std::collections::HashSet;
use tract_tensorflow::prelude::*;

fn main() -> TractResult<()> {
    // This is the range of dog ids in `imagenet_slim_labels.txt`
    // There's probably a better way to do this in tensorflow
    let dog_class_set: HashSet<u16> = (153..=277).collect();

    let model = tract_tensorflow::tensorflow()
        // load the model
        .model_for_path("models/mobilenet_v2_1.4_224_frozen.pb")?
        // specify input type and shape
        .with_input_fact(0, f32::fact(&[1, 224, 224, 3]).into())?
        // optimize the model
        .into_optimized()?
        // make the model runnable and fix its inputs and outputs
        .into_runnable()?;

    let cam = camera_capture::create(0)?;
    let cam = cam.fps(30.0).unwrap().start()?;

    for frame in cam {
        // create RgbImage from Frame, resize it and make a Tensor out of it
        let image =
            image::RgbImage::from_raw(frame.width(), frame.height(), frame.to_vec()).unwrap();
        let resized =
            image::imageops::resize(&image, 224, 224, ::image::imageops::FilterType::Triangle);
        let image: Tensor =
            tract_ndarray::Array4::from_shape_fn((1, 224, 224, 3), |(_, y, x, c)| {
                resized[(x as _, y as _)][c] as f32 / 255.0
            })
            .into();

        // run the model on the input
        let result = model.run(tvec!(image))?;

        // find and display the max value with its index
        let best = result[0]
            .to_array_view::<f32>()?
            .iter()
            .cloned()
            .zip(1..)
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        match best {
            Some((_, class_idx)) => {
                if dog_class_set.contains(&class_idx) {
                    println!("\n\ndog detected: {}\n\n", &class_idx);
                }
            }
            None => {
                println!("no match");
            }
        }

        println!("best: {:?}", best);
    }

    Ok(())
}
