use crate::image_processing::FrameBuffer;
use ndarray::prelude::*;
use std::collections::HashSet;
use std::error::Error;
use tensorflow::{Graph, Session, SessionRunArgs, Tensor};

#[derive(Debug, Clone, Copy)]
pub struct Detector<'a> {
    pub graph: &'a Graph,
    pub session: &'a Session,
    pub match_set: &'a HashSet<u32>,
}

impl<'a> Detector<'a> {
    pub fn new(graph: &'a Graph, session: &'a Session, match_set: &'a HashSet<u32>) -> Self {
        Detector {
            graph,
            session,
            match_set,
        }
    }

    pub fn detect(&self, frame_buffer: &FrameBuffer) -> Result<bool, Box<dyn Error>> {
        let (width, height) = frame_buffer.dimensions();
        let image_tensor = self
            .graph
            .operation_by_name_required("image_tensor")
            .unwrap();

        // TODO: handle unwraps
        // Can I just use the Vec from the ImageBuffer?
        let image_array = Array::from_shape_vec((640, 480, 3), frame_buffer.to_vec()).unwrap();
        // let image_array = Array::from_shape_vec((640, 480, 3), resized_image.to_vec()).unwrap();
        let image_array_expanded = image_array.insert_axis(Axis(0));

        let input_image_tensor = Tensor::new(&[1, height as u64, width as u64, 3])
            .with_values(image_array_expanded.as_slice().unwrap())
            .unwrap();

        let mut step = SessionRunArgs::new();

        step.add_feed(&image_tensor, 0, &input_image_tensor);

        let classes = self
            .graph
            .operation_by_name_required("detection_classes")
            .unwrap();
        let classes_token = step.request_fetch(&classes, 0);

        self.session.run(&mut step).unwrap();

        // let num_detections_tensor = step.fetc[h::<f32>(num_detections_token).unwrap();
        // println!("num_detections_tensor {:?}", num_detections_tensor[0]);

        let classes_tensor = step.fetch::<f32>(classes_token).unwrap();
        let classes = classes_tensor.iter().map(|x| *x as u32).collect::<Vec<_>>();
        let top_classification = classes[0] as u32;

        Ok(self.match_set.contains(&top_classification))
    }
}
