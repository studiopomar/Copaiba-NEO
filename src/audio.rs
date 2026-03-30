// audio.rs — WAV file loading using hound
use hound::WavReader;
use std::path::Path;
use std::sync::Arc;
use crate::spectrogram::SpectrogramData;

#[derive(Clone)]
pub struct WavData {
    pub samples: Arc<Vec<f32>>, // normalized -1.0..1.0, mono mix
    pub sample_rate: u32,
    pub duration_ms: f64,
    pub max_amplitude: f32,
}

#[derive(Clone)]
pub struct PitchData {
    pub times: Vec<f64>,
    pub frequencies: Vec<f64>,
}

/// Pre-computed spectrogram and pitch data (not Clone because it's large)
pub struct WavWithSpec {
    pub wav: WavData,
    pub spec_data: Option<SpectrogramData>,
    pub pitch_data: Option<PitchData>,
}

/// Load a WAV file from a file path
pub fn load_wav(path: &Path) -> Result<WavWithSpec, String> {
    let reader = WavReader::open(path).map_err(|e| e.to_string())?;
    load_wav_from_reader(reader)
}

/// Load a WAV file from raw bytes (embedded)
pub fn load_wav_from_bytes(bytes: &[u8]) -> Result<WavWithSpec, String> {
    let reader = WavReader::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;
    load_wav_from_reader(reader)
}

fn load_wav_from_reader<R: std::io::Read + std::io::Seek>(mut reader: WavReader<R>) -> Result<WavWithSpec, String> {
    let spec = reader.spec();
    let sample_rate = spec.sample_rate;
    
    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().map(|s| s.unwrap_or(0.0)).collect(),
        hound::SampleFormat::Int => {
            let max = (1 << (spec.bits_per_sample - 1)) as f32;
            reader.samples::<i32>().map(|s| s.unwrap_or(0) as f32 / max).collect()
        }
    };

    // Mix to mono if needed
    let channels = spec.channels;
    let mono: Vec<f32> = if channels > 1 {
        samples.chunks(channels as usize)
            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
            .collect()
    } else {
        samples
    };

    let duration_ms = (mono.len() as f64 / sample_rate as f64) * 1000.0;
    
    // Find max amplitude for visual normalization
    let mut max_abs = 0.0f32;
    for &s in &mono {
        let abs = s.abs();
        if abs > max_abs {
            max_abs = abs;
        }
    }
    if max_abs == 0.0 {
        max_abs = 1.0;
    }

    let samples_arc = Arc::new(mono);

    Ok(WavWithSpec {
        wav: WavData {
            samples: samples_arc,
            sample_rate,
            duration_ms,
            max_amplitude: max_abs,
        },
        spec_data: None,
        pitch_data: None,
    })
}
