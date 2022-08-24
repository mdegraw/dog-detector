mod detector;

use detector::Detector;
use image::bmp::BmpEncoder;
use nokhwa::{Camera, CameraFormat, FrameFormat};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::collections::HashSet;
use std::time::Duration;
use tokio::task;
use tract_tensorflow::prelude::*;

static DOG_DETECTION_TOPIC: &str = "house/front_door/dog_detection";

#[tokio::main]
async fn main() -> TractResult<()> {
    // This is the range of dog ids in `imagenet_slim_labels.txt`
    // There's probably a better way to do this in tensorflow
    let dog_class_set: HashSet<u16> = (153..=277).collect();

    // TODO: pull connection strings from env
    let mut mqttoptions = MqttOptions::new("test-d322", "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    client
        .subscribe(DOG_DETECTION_TOPIC, QoS::AtMostOnce)
        .await?;

    // poll the event loop
    task::spawn(async move {
        loop {
            let _event = &eventloop.poll().await;
            // println!("{:?}", event.unwrap());
        }
    });

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
    let mut detected_count = 0;

    let mut camera = Camera::new(
        0,
        Some(CameraFormat::new_from(640, 480, FrameFormat::MJPEG, 30)),
    )?;
    camera.open_stream()?;

    loop {
        let frame_buffer = camera.frame()?;

        let clone = client.clone();
        detected_count += dog_detector.detect(&frame_buffer, 0.2)? as u32;

        println!("detected_count: {}", detected_count);

        if detected_count > 12 {
            let now = chrono::offset::Local::now();
            println!("\n\ndog detected at {:?}\n\n", now);

            let image = image::GrayImage::from_raw(
                frame_buffer.width(),
                frame_buffer.height(),
                frame_buffer.to_vec(),
            )
            .unwrap();
            let resized =
                image::imageops::resize(&image, 128, 64, image::imageops::FilterType::Triangle);

            let mut buf = Vec::new();
            let mut encoder = BmpEncoder::new(&mut buf);
            let data: &[u8] = &resized.as_raw();

            encoder.encode(
                data,
                resized.width(),
                resized.height(),
                image::ColorType::L8,
            )?;

            let res: &[u8] = &buf;
            let img = res.to_vec();

            task::spawn(async move {
                println!("\n\nPublishing message\n\n");
                clone
                    .publish(
                        DOG_DETECTION_TOPIC,
                        QoS::AtLeastOnce,
                        false,
                        // String::from("hello!"),
                        img.to_vec(),
                    )
                    .await
                    .unwrap();
            });

            break;
        }
    }

    Ok(())
}
//
