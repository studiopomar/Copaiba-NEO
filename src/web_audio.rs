#![cfg(target_arch = "wasm32")]

use std::cell::RefCell;
use wasm_bindgen::JsCast;
use web_sys::{AudioBufferSourceNode, AudioContext};

thread_local! {
    static CURRENT: RefCell<Option<AudioBufferSourceNode>> = const { RefCell::new(None) };
}

pub fn stop() {
    CURRENT.with(|current| {
        if let Some(source) = current.borrow_mut().take() {
            let _ = source.stop();
        }
    });
}

pub fn play(samples: &[f32], sample_rate: u32) -> Result<(), String> {
    stop();
    let context = AudioContext::new().map_err(|e| format!("AudioContext: {e:?}"))?;
    let buffer = context.create_buffer(1, samples.len() as u32, sample_rate as f32)
        .map_err(|e| format!("AudioBuffer: {e:?}"))?;
    buffer.copy_to_channel(samples, 0).map_err(|e| format!("AudioBuffer data: {e:?}"))?;
    let source = context.create_buffer_source().map_err(|e| format!("AudioSource: {e:?}"))?;
    source.set_buffer(Some(&buffer));
    source.connect_with_audio_node(&context.destination()).map_err(|e| format!("Audio connect: {e:?}"))?;
    source.start().map_err(|e| format!("Audio start: {e:?}"))?;
    CURRENT.with(|current| *current.borrow_mut() = Some(source));
    Ok(())
}
