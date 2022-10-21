extern crate tensorflow;

mod context;
mod detector;
mod image_processing;

use context::{Context, DetectionState};
use detector::Detector;
use image_processing::image_buffer_to_oled_byte_array;
use nokhwa::{Camera, CameraFormat, FrameFormat};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, Publish, QoS};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::result::Result;
use std::sync::{Arc, Mutex};
use tensorflow::{Graph, ImportGraphDefOptions, Session, SessionOptions};
use tokio::{
    task,
    time::{Duration, Instant},
};

const DOG_DETECTION_TOPIC: &str = "house/front_door/dog_detection";
const DOG_DETECTION_STREAM_TOPIC: &str = "house/front_door/dog_detection/stream";
const DOG_DETECTION_STREAM_END_TOPIC: &str = "house/front_door/dog_detection/stream/end";
const DOG_DETECTION_ACKNOWLEDGE_TOPIC: &str = "house/front_door/dog_detection/acknowledge";

// TODO: Load from input arg path
const MODEL: &str = "/home/userone/Devel/dog-detector/tensorflow/models/ssd_mobilenet_v1_coco_2017_11_17/frozen_inference_graph.pb";

fn extract_from_event(event: &Event) -> Option<&Publish> {
    match event {
        Event::Incoming(incoming) => match incoming {
            Packet::Publish(incoming_pub) => Some(incoming_pub),
            _ => None,
        },
        _ => None,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // This is the set of COCO categories we want to match on
    let match_set: HashSet<u32> = HashSet::from([18]);

    let mut context = Arc::new(Mutex::new(Context::new(
        Duration::new(30, 0),
        Duration::new(90, 0),
        5,
    )));

    // TODO: pull connection strings from env
    let mut mqttoptions = MqttOptions::new("dog-detection", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    client
        .subscribe(DOG_DETECTION_TOPIC, QoS::AtMostOnce)
        .await?;

    client
        .subscribe(DOG_DETECTION_ACKNOWLEDGE_TOPIC, QoS::AtMostOnce)
        .await?;

    let thread_context = context.clone();

    // we want this outside of the main detection loop
    // poll the event loop and update the context
    task::spawn(async move {
        loop {
            let event = &eventloop.poll().await.unwrap();

            // handle incoming event
            if let Some(incoming_message) = extract_from_event(event) {
                match incoming_message.topic.as_str() {
                    DOG_DETECTION_ACKNOWLEDGE_TOPIC => {
                        println!("INCOMING MESSAGE TOPIC: {}", incoming_message.topic);
                        println!("INCOMING MESSAGE PAYLOAD: {:?}", incoming_message.payload);

                        if let Ok(mut ctx) = thread_context.lock() {
                            let now = Instant::now();
                            ctx.detected_count = 0;
                            ctx.state = DetectionState::Paused(now);
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    let mut graph = Graph::new();
    let mut proto = Vec::new();

    // TODO: add error handling
    File::open(MODEL).unwrap().read_to_end(&mut proto).unwrap();

    graph
        .import_graph_def(&proto, &ImportGraphDefOptions::new())
        .unwrap();

    let session = Session::new(&SessionOptions::new(), &graph).unwrap();

    let dog_detector = Detector::new(&graph, &session, &match_set, 0.2);

    let mut camera = Camera::new(
        // TODO: set this as input arg and allow use of the virtual camera we created
        0,
        Some(CameraFormat::new_from(640, 480, FrameFormat::MJPEG, 30)),
    )?;
    camera.open_stream().expect("Could not open camera stream");

    let thread_context = context.clone();

    loop {
        let frame_buffer = camera.frame()?;

        let clone = client.clone();

        if let Ok(mut ctx) = thread_context.lock() {
            match ctx.state {
                DetectionState::Streaming(_) | DetectionState::Paused(_) => {}
                _ => {
                    let is_detected = dog_detector
                        .detect(&frame_buffer)
                        .expect("Error running detection model");

                    if is_detected {
                        let now = Instant::now();
                        let clone = client.clone();
                        if ctx.is_detected() {
                            task::spawn(async move {
                                clone
                                    .publish(DOG_DETECTION_TOPIC, QoS::AtLeastOnce, false, [])
                                    .await
                                    .unwrap();
                            });
                        }
                        ctx.state = DetectionState::Detected(now);
                    }
                }
            }

            match ctx.next() {
                DetectionState::Streaming(_) => {
                    // let byte_array = image_buffer_to_oled_byte_array(&frame_buffer, 37);
                    // let byte_array = image_buffer_to_oled_byte_array(&frame_buffer, 44);
                    let byte_array = image_buffer_to_oled_byte_array(&frame_buffer, 28);

                    task::spawn(async move {
                        clone
                            .publish(
                                DOG_DETECTION_STREAM_TOPIC,
                                QoS::AtLeastOnce,
                                false,
                                byte_array,
                            )
                            .await
                            .unwrap();
                    });
                }
                // DetectionState::StreamEnd => {
                //     task::spawn(async move {
                //         clone
                //             .publish(DOG_DETECTION_STREAM_END_TOPIC, QoS::AtLeastOnce, false, [])
                //             .await
                //             .unwrap();
                //     });
                // }
                _ => {}
            }
        }
    }

    Ok(())
}
