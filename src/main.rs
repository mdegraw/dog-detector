extern crate camera_capture;
mod detector;

use detector::Detector;
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

    let dog_detector = Detector::new(&model, &dog_class_set);

    let cam = camera_capture::create(0)?;
    let cam = cam.fps(30.0).unwrap().start()?;

    for frame_buffer in cam {
        if let Ok(dog_detected) = dog_detector.detect(&frame_buffer) {
            println!("\n\ndog detected: {}\n\n", dog_detected);
        }
    }

    Ok(())
}
