use hound::WavSpec;
use std::path::PathBuf;

pub struct Sample {
    data: Vec<f32>,
    file_path: PathBuf,
    spec: WavSpec,
}

impl Sample {
    pub fn load<P: Into<PathBuf>>(path: P) -> std::io::Result<Sample> {
        todo!()
    }
}
