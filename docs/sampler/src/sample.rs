use hound::WavSpec;
use std::path::{Path, PathBuf};

pub struct Sample {
    data: Vec<f32>,
    file_path: PathBuf,
    spec: WavSpec,
}

impl Sample {
    fn load<P: AsRef<Path>>(path: P) -> std::io::Result<Sample> {
        todo!()
    }
}
