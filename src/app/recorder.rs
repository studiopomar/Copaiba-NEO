use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use egui_i18n::tr;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

pub fn start_recording(
    samples: Arc<Mutex<Vec<f32>>>,
    stop_signal: Arc<AtomicBool>,
) -> Result<(cpal::Stream, u32), String> {
    let host = cpal::default_host();
    let device = host.default_input_device()
        .ok_or(tr!("recorder.error.no_device"))?;

    let config = device.default_input_config()
        .map_err(|e| format!("{} {}", tr!("recorder.error.config"), e))?;
    
    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;

    let samples_clone = samples.clone();
    let stop_signal_clone = stop_signal.clone();

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &_| {
                if !stop_signal_clone.load(Ordering::SeqCst) {
                    let mut s = samples_clone.lock().unwrap();
                    // Converte para mono se necessário (média dos canais)
                    if channels > 1 {
                        for frame in data.chunks_exact(channels) {
                            let sum: f32 = frame.iter().sum();
                            s.push(sum / channels as f32);
                        }
                    } else {
                        s.extend_from_slice(data);
                    }
                }
            },
            |err| eprintln!("{} {}", tr!("recorder.error.stream"), err),
            None,
        ),
        cpal::SampleFormat::I16 => device.build_input_stream(
            &config.into(),
            move |data: &[i16], _: &_| {
                if !stop_signal_clone.load(Ordering::SeqCst) {
                    let mut s = samples_clone.lock().unwrap();
                    if channels > 1 {
                        for frame in data.chunks_exact(channels) {
                            let sum: f32 = frame.iter().map(|&x| x as f32 / 32768.0).sum();
                            s.push(sum / channels as f32);
                        }
                    } else {
                        for &sample in data {
                            s.push(sample as f32 / 32768.0);
                        }
                    }
                }
            },
            |err| eprintln!("{} {}", tr!("recorder.error.stream"), err),
            None,
        ),
        _ => return Err(tr!("recorder.error.sample_format").to_string()),
    }.map_err(|e| format!("{} {}", tr!("recorder.error.create_stream"), e))?;

    stream.play().map_err(|e| format!("{} {}", tr!("recorder.error.play_stream"), e))?;
    Ok((stream, sample_rate))
}
