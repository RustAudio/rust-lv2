use crate::sample::Sample;
use lv2::lv2_urid::LV2Map;
use lv2::lv2_worker::{ResponseHandler, Schedule, Worker, WorkerError};
use lv2::prelude::*;
use std::path::PathBuf;

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

#[uri("https://github.com/RustAudio/rust-lv2/tree/master/docs/sampler")]
pub struct Sampler {
    current_sample: Option<Sample>,
}

impl<'a> Plugin<'a> for Sampler {
    type Ports = ();
    type Features = ();

    fn new(plugin_info: &PluginInfo, features: Self::Features) -> Option<Self> {
        todo!()
    }

    fn run(&mut self, ports: &mut Self::Ports, sample_count: u32) {
        todo!()
    }
}

pub enum WorkerRequest {
    LoadSample(PathBuf),
    FreeSample(Sample),
}

impl<'a> Worker<'a> for Sampler {
    type RequestData = WorkerRequest;
    type ResponseData = Option<Sample>;

    fn work(
        response_handler: &ResponseHandler<Self>,
        data: Self::RequestData,
    ) -> Result<(), WorkerError> {
        match data {
            WorkerRequest::LoadSample(path) => {
                let sample = Sample::load(path).map_err(|e| {
                    eprintln!("{:?}", e);
                    WorkerError::Unknown
                })?;

                response_handler.respond(Some(sample)).unwrap(); // FIXME
            }
            WorkerRequest::FreeSample(_) => {} // just drop it
        }

        Ok(())
    }

    fn work_response(&mut self, data: Self::ResponseData) -> Result<(), WorkerError> {
        let new_sample = if let Some(data) = data {
            data
        } else {
            return Ok(());
        };
        let previous_sample = self.current_sample.replace(new_sample);

        if let Some(previous_sample) = previous_sample {
            // TODO
        }

        Ok(())
    }
}
