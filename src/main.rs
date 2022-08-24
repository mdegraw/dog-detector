extern crate camera_capture;
mod detector;

use detector::Detector;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::collections::HashSet;
use std::error::Error;
use std::time::Duration;
use tokio::{task, time};

use tract_tensorflow::prelude::*;

#[tokio::main]
async fn main() -> TractResult<()> {
    // This is the range of dog ids in `imagenet_slim_labels.txt`
    // There's probably a better way to do this in tensorflow
    let dog_class_set: HashSet<u16> = (153..=277).collect();

    let mut mqttoptions = MqttOptions::new("test-d322", "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client
        .subscribe("house/front_door/dog_detection", QoS::AtMostOnce)
        .await?; //unwrap();

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
        let clone = client.clone();
        let detected = dog_detector.detect(&frame_buffer, 0.2)?;
        if detected {
            let now = chrono::offset::Local::now();
            println!("\n\ndog detected at {:?}\n\n", now);

            task::spawn(async move {
                println!("\n\nPublishing message\n\n");
                clone
                    .publish(
                        "house/front_door/dog_detection",
                        QoS::AtLeastOnce,
                        false,
                        frame_buffer.to_vec(),
                    )
                    .await
                    .unwrap();
            });
        }
    }

    while let Ok(notification) = eventloop.poll().await {
        println!("Received = {:?}", notification);
    }

    Ok(())
}
//
