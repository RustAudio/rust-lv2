use crate::sample::Sample;
use lv2::lv2_urid::LV2Map;
use lv2::lv2_worker::{ResponseHandler, Schedule, Worker, WorkerError};
use lv2::prelude::*;
use std::path::Path;

mod sample;

pub struct SamplerFeatures<'a> {
    map: LV2Map<'a>,
    schedule: Schedule<'a, Sampler>,
}

struct SamplerPorts {
    control: InputPort<Control>,
    notify: OutputPort<AtomPort>,
    output: OutputPort<Audio>,
}

pub struct Sampler {}

enum WorkType {
    LoadSample(Path),
}

impl Worker for Sampler {
    type WorkData = ();
    type ResponseData = Sample;

    fn work(
        response_handler: &ResponseHandler<Self>,
        data: Self::WorkData,
    ) -> Result<(), WorkerError> {
        response_handler.respond(())?;
        todo!()
    }
}
