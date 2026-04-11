/// wsola.rs — Waveform Similarity Overlap-Add for Time-Stretching
/// Pure Rust implementation of high-quality time stretching without pitch shift.

pub fn wsola_stretch(samples: &[f32], speed: f32) -> Vec<f32> {
    if (speed - 1.0).abs() < 0.01 || speed <= 0.0 {
        return samples.to_vec();
    }

    let window_size = 1024;
    let synthesis_hop = 512;
    let analysis_hop = ((synthesis_hop as f32 * speed) as usize).max(1);
    let _search_range = 256; // range to look for alignment

    let mut output = Vec::with_capacity((samples.len() as f32 / speed) as usize + window_size);
    
    // Create Hanning window
    let mut window = vec![0.0f32; window_size];
    for i in 0..window_size {
        window[i] = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (window_size - 1) as f32).cos());
    }

    // First grain
    let mut last_best_pos;
    let mut current_analysis_pos;
    if samples.len() >= window_size {
        output.extend_from_slice(&samples[0..window_size]);
        last_best_pos = 0;
        current_analysis_pos = analysis_hop;
    } else {
        return samples.to_vec();
    }

    let mut synthesis_pos = synthesis_hop;

    while current_analysis_pos + window_size + _search_range < samples.len() {
        // Find best match in search range
        let mut best_offset = 0;
        let mut min_diff = f32::MAX;
        
        let target_pos = last_best_pos + synthesis_hop;
        let search_start = current_analysis_pos.saturating_sub(_search_range / 2);
        let _search_end = current_analysis_pos + (_search_range / 2);

        for offset in 0.._search_range {
            let pos = search_start + offset;
            if pos + window_size >= samples.len() || target_pos + window_size >= samples.len() { break; }
            
            // Calculate similarity (Sum of Absolute Differences)
            let mut diff = 0.0f32;
            // Sampling for speed
            for i in (0..window_size).step_by(8) {
                diff += (samples[target_pos + i] - samples[pos + i]).abs();
            }
            
            if diff < min_diff {
                min_diff = diff;
                best_offset = offset;
            }
        }

        let best_pos = search_start + best_offset;
        
        // Ensure output has enough space
        if output.len() < synthesis_pos + window_size {
            output.resize(synthesis_pos + window_size, 0.0);
        }

        // Overlap and add with crossfade
        for i in 0..window_size {
            let synth_i = synthesis_pos + i;
            if synth_i >= output.len() { break; }
            
            // Linear crossfade or Hann window merge
            if i < synthesis_hop {
                // Crossfade region
                let fade = i as f32 / synthesis_hop as f32;
                output[synth_i] = output[synth_i] * (1.0 - fade) + samples[best_pos + i] * fade;
            } else {
                // New samples region
                output[synth_i] = samples[best_pos + i];
            }
        }

        last_best_pos = best_pos;
        current_analysis_pos += analysis_hop;
        synthesis_pos += synthesis_hop;
    }

    output.truncate(synthesis_pos + synthesis_hop);
    output
}
