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
    pub threshold: f32,
}

impl<'a> Detector<'a> {
    pub fn new(
        graph: &'a Graph,
        session: &'a Session,
        match_set: &'a HashSet<u32>,
        threshold: f32,
    ) -> Self {
        Detector {
            graph,
            session,
            match_set,
            /// threshold is a number between 0 and 1
            threshold,
        }
    }

    pub fn detect(&self, frame_buffer: &FrameBuffer) -> Result<bool, Box<dyn Error>> {
        let (width, height) = frame_buffer.dimensions();
        let image_tensor = self.graph.operation_by_name_required("image_tensor")?;

        let image_arr = Array::from_shape_vec((640, 480, 3), frame_buffer.to_vec())?;
        let image_arr_base = image_arr.insert_axis(Axis(0));
        let image_arr_slice = image_arr_base.as_slice();
        let image_arr_expanded = match image_arr_slice {
            Some(arr) => arr,
            None => &[],
        };

        let input_image_tensor =
            Tensor::new(&[1, height as u64, width as u64, 3]).with_values(image_arr_expanded)?;

        let mut step = SessionRunArgs::new();

        step.add_feed(&image_tensor, 0, &input_image_tensor);

        let classes = self.graph.operation_by_name_required("detection_classes")?;
        let classes_token = step.request_fetch(&classes, 0);
        let scores = self.graph.operation_by_name_required("detection_scores")?;

        let scores_token = step.request_fetch(&scores, 0);

        self.session.run(&mut step)?;

        let classes_tensor = step.fetch::<f32>(classes_token)?;
        let scores_tensor = step.fetch::<f32>(scores_token)?;
        let results = classes_tensor
            .iter()
            .map(|x| *x as u32)
            .zip(scores_tensor.iter().map(|x| *x))
            .filter(|(class, score)| self.match_set.contains(class) && *score > self.threshold)
            .collect::<Vec<_>>();

        Ok(results.len() > 0)
    }
}
