extern crate tensorflow;

mod context;
mod detector;
mod image_processing;

use context::{Context, DetectionState, PauseState};
use detector::Detector;
use image_processing::image_buffer_to_oled_byte_array;
use nokhwa::{Camera, CameraFormat, FrameFormat};
use rumqttc::{AsyncClient, Event, MqttOptions, QoS};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::result::Result;
use tensorflow::{Graph, ImportGraphDefOptions, Session, SessionOptions};
use tokio::{
    task,
    time::{Duration, Instant},
};

static DOG_DETECTION_TOPIC: &str = "house/front_door/dog_detection";
// TODO: Load from input arg path
const MODEL: &str = "";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // This is the set of COCO categories we want to match on
    let match_set: HashSet<u32> = HashSet::from([18]);

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

    let mut graph = Graph::new();
    let mut proto = Vec::new();

    // TODO: add error handling
    File::open(MODEL).unwrap().read_to_end(&mut proto).unwrap();

    graph
        .import_graph_def(&proto, &ImportGraphDefOptions::new())
        .unwrap();

    let session = Session::new(&SessionOptions::new(), &graph).unwrap();

    let dog_detector = Detector::new(&graph, &session, &match_set);

    let mut camera = Camera::new(
        // TODO: set this as input arg
        // We're using the virtual camera we created
        1,
        // 2,
        Some(CameraFormat::new_from(640, 480, FrameFormat::MJPEG, 30)),
    )?;
    camera.open_stream().expect("Could not open camera stream");

    // let mut detected_time: Option<tokio::time::Instant> = None;

    loop {
        let frame_buffer = camera.frame()?;

        let clone = client.clone();

        let is_detected = dog_detector
            .detect(&frame_buffer)
            .expect("Error running detection model");

        let state = if is_detected {
            let now = Instant::now();
            DetectionState::Detected(now)
        } else {
            context.state
        };

        match context.next(state) {
            DetectionState::Paused(PauseState::OledStreaming) => {
                // let byte_array = image_buffer_to_oled_byte_array(&frame_buffer, 37);
                // let byte_array = image_buffer_to_oled_byte_array(&frame_buffer, 44);
                let byte_array = image_buffer_to_oled_byte_array(&frame_buffer, 28);

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
