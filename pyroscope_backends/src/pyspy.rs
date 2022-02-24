use super::error::Result;
use super::types::{Backend, State};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use py_spy::config::Config;
use py_spy::sampler::Sampler;

#[derive(Default)]
pub struct Pyspy {
    state: State,
    buffer: Arc<Mutex<HashMap<String, usize>>>,
}

impl std::fmt::Debug for Pyspy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pyspy Backend")
    }
}

impl Backend for Pyspy {
    fn get_state(&self) -> State {
        self.state
    }

    fn initialize(&mut self, sample_rate: i32) -> Result<()> {
        //let buffer = Some(Arc::new(Mutex::new(String::new())));

        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        let mut buffer = self.buffer.clone();

        std::thread::spawn(move || {
            let mut config = Config::default();
            config.subprocesses = true;
            let sampler = Sampler::new(1508076, &config).unwrap();
            for sample in sampler {
                for trace in sample.traces {
                    let frame = trace
                        .frames
                        .iter()
                        .rev()
                        .map(|frame| {
                            let filename = match &frame.short_filename {
                                Some(f) => &f,
                                None => &frame.filename,
                            };
                            if frame.line != 0 {
                                format!("{} ({}:{})", frame.name, filename, frame.line)
                            } else if filename.len() > 0 {
                                format!("{} ({})", frame.name, filename)
                            } else {
                                frame.name.clone()
                            }
                        })
                        .collect::<Vec<String>>()
                        .join(";");
                    *buffer.lock().unwrap().entry(frame).or_insert(0) += 1;
                }
            }
        });

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn report(&mut self) -> Result<Vec<u8>> {
        let mut buffer = self.buffer.clone();

        let col: Vec<String> = buffer
            .lock()
            .unwrap()
            .iter()
            .map(|(k, v)| format!("{} {}", k, v))
            .collect();

        let v8: Vec<u8> = col.join("\n").into_bytes();

        buffer.lock().unwrap().clear();

        Ok(v8)
    }
}
