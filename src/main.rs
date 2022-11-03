extern crate tensorflow;

mod config;
mod context;
mod detector;
mod image_processing;

use config::Config;
use context::{Context, DetectionState};
use detector::Detector;
use directories::ProjectDirs;
use image_processing::image_buffer_to_oled_byte_array;
use nokhwa::{Camera, CameraFormat, FrameFormat};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, Publish, QoS};
use std::collections::HashSet;
use std::error::Error;
use std::fs::{read_to_string, File};
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
const DOG_DETECTION_ACKNOWLEDGE_TOPIC: &str = "house/front_door/dog_detection/acknowledge";

fn extract_from_event(event: &Event) -> Option<&Publish> {
    match event {
        Event::Incoming(incoming) => match incoming {
            Packet::Publish(incoming_pub) => Some(incoming_pub),
            _ => None,
        },
        _ => None,
    }
}

#[allow(unreachable_code)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let Config {
        mqtt_host,
        mqtt_port,
        mqtt_username,
        mqtt_password,
        detector_threshold,
        stream_duration,
        pause_duration,
        camera_index,
        camera_fps,
        oled_threshold,
        tensorflow_model_file,
    } = if let Some(proj_dirs) = ProjectDirs::from("dev", "odo", "dog-detector") {
        let config_dir = proj_dirs.config_dir();
        let config_file = read_to_string(config_dir.join("config.toml"));

        match config_file {
            Ok(file) => toml::from_str(&file)?,
            Err(_) => panic!("Must provide config file!"),
        }
    } else {
        panic!("Must place config.toml in the correct directory");
    };

    // This is the set of COCO categories we want to match on
    let match_set: HashSet<u32> = HashSet::from([18]);

    let context = Arc::new(Mutex::new(Context::new(
        Duration::new(stream_duration, 0),
        Duration::new(pause_duration, 0),
        5,
    )));

    let mut mqttoptions = MqttOptions::new("dog-detection", mqtt_host, mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    if let (Some(mqtt_username), Some(mqtt_password)) = (mqtt_username, mqtt_password) {
        mqttoptions.set_credentials(mqtt_username, mqtt_password);
    }

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

    File::open(tensorflow_model_file)?
        .read_to_end(&mut proto)
        .expect("Error opening tensorflow model file.");

    graph.import_graph_def(&proto, &ImportGraphDefOptions::new())?;

    let session = Session::new(&SessionOptions::new(), &graph)?;

    let dog_detector = Detector::new(&graph, &session, &match_set, detector_threshold);

    let mut camera = Camera::new(
        camera_index,
        Some(CameraFormat::new_from(
            640,
            480,
            FrameFormat::MJPEG,
            camera_fps,
        )),
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
                    let byte_array = image_buffer_to_oled_byte_array(&frame_buffer, oled_threshold);

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
                _ => {}
            }
        }
    }

    Ok(())
}
