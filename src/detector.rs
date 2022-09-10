use crate::image_processing::FrameBuffer;
use std::collections::HashSet;
use tract_tensorflow::prelude::*;
use tract_tensorflow::tract_core::anyhow;

type TfModel = SimplePlan<TypedFact, Box<dyn TypedOp>, TypedModel>;

#[derive(Debug, Clone, Copy)]
pub struct Detector<'a> {
    pub model: &'a TfModel,
    pub match_set: &'a HashSet<u16>,
}

impl<'a> Detector<'a> {
    pub fn new(model: &'a TfModel, match_set: &'a HashSet<u16>) -> Self {
        Detector { model, match_set }
    }

    pub fn detect(
        &self,
        frame_buffer: &FrameBuffer,
        confidence_threshold: f32,
    ) -> Result<bool, anyhow::Error> {
        let resized_image =
            image::imageops::resize(frame_buffer, 224, 224, image::imageops::FilterType::Nearest);

        let image: Tensor =
            tract_ndarray::Array4::from_shape_fn((1, 224, 224, 3), |(_, y, x, c)| {
                resized_image[(x as _, y as _)][c] as f32 / 255.0
            })
            .into();

        // TODO: check to see if multiple classes are returned
        // run the model on the input
        let result = self.model.run(tvec!(image))?;

        let is_detected = result[0]
            .to_array_view::<f32>()?
            .iter()
            .cloned()
            .zip(1..)
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(confidence, class_idx)| {
                self.match_set.contains(&class_idx) && confidence > confidence_threshold
            });

        match is_detected {
            Some(val) => Ok(val),
            None => Ok(false),
        }
    }
}
