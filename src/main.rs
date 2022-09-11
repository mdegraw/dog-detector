mod context;
mod detector;
mod image_processing;

// use context::{Context, Event, DetectionState, PauseState};
use context::{Context, DetectionState, PauseState};
use detector::Detector;
use image_processing::image_buffer_to_oled_byte_array;
use nokhwa::{Camera, CameraFormat, FrameFormat, NetworkCamera};
use rumqttc::{AsyncClient, Event, MqttOptions, QoS};
use std::collections::HashSet;
// use std::time::Duration;
use tokio::{
    task,
    time::{Duration, Instant},
};
use tract_tensorflow::prelude::*;

static DOG_DETECTION_TOPIC: &str = "house/front_door/dog_detection";

#[tokio::main]
async fn main() -> TractResult<()> {
    // This is the range of dog ids in `imagenet_slim_labels.txt`
    // There's probably a better way to do this in tensorflow
    let dog_class_set: HashSet<u16> = (153..=277).collect();

    let mut context = Context::new(Duration::new(30, 0), 5);

    // TODO: pull connection strings from env
    let mut mqttoptions = MqttOptions::new("test-d322", "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    client
        .subscribe(DOG_DETECTION_TOPIC, QoS::AtMostOnce)
        .await?;

    // poll the event loop
    task::spawn(async move {
        loop {
            // let event = &eventloop.poll().await.unwrap();
            match &eventloop.poll().await {
                Ok(event) => match event {
                    Event::Incoming(incoming) => {
                        // println!("incoming {:?}", incoming);
                    }
                    Event::Outgoing(outgoing) => {
                        // println!("outgoing {:?}", outgoing);
                    }
                },
                Err(_) => {}
            };
            // println!("{:?}", event.unwrap());
        }
    });

    let model = tract_tensorflow::tensorflow()
        // load the model
        .model_for_path("tensorflow/models/mobilenet_v2_1.4_224_frozen.pb")?
        // specify input type and shape
        .with_input_fact(0, f32::fact(&[1, 224, 224, 3]).into())?
        // optimize the model
        .into_optimized()?
        // make the model runnable and fix its inputs and outputs
        .into_runnable()?;

    let dog_detector = Detector::new(&model, &dog_class_set);

    let mut camera = Camera::new(
        // TODO: set this as input arg
        // We're using the virtual camera we created
        2,
        Some(CameraFormat::new_from(640, 480, FrameFormat::MJPEG, 30)),
    )?;
    camera.open_stream().expect("Could not open camera stream");

    // let mut detected_time: Option<tokio::time::Instant> = None;

    loop {
        let frame_buffer = camera.frame()?;

        let clone = client.clone();

        let is_detected = dog_detector
            .detect(&frame_buffer, 0.2)
            .expect("Error running detection model");

        let state = if is_detected {
            let now = Instant::now();
            DetectionState::Detected(now)
        } else {
            context.state
        };

        match context.next(state) {
            DetectionState::Paused(PauseState::OledStreaming) => {
                let byte_array = image_buffer_to_oled_byte_array(&frame_buffer, 37);
                // let byte_array = image_buffer_to_oled_byte_array(&frame_buffer, 44);

                task::spawn(async move {
                    clone
                        .publish(DOG_DETECTION_TOPIC, QoS::AtLeastOnce, false, byte_array)
                        .await
                        .unwrap();
                });
            }
            _ => {}
        }
    }

    Ok(())
}
