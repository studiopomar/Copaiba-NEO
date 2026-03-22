# Copaiba Mini

**Copaiba Mini** is a lite, high-performance oto.ini editor for UTAU voicebanks, written in Rust. It focuses on visual precision, speed, and modern user experience.

## Features

- **High-Fidelity Visuals**: 
  - Spectrogram with logarithmic frequency scale and bicubic interpolation.
  - Waveform with continuous spline rendering and peak envelopes.
- **Precision Editing**: 
  - Sub-millisecond parameter editing with **Copaiba (QWERT)**, **SetParam (F1-F5)**, or **Custom** keyboard profiles.
  - SRP (Snap Relative to Preutterance) and SRnA (Snap Relative to Nothing) modes for smart offset adjustments.
  - Excel-style grid for bulk editing, multi-selection (Ctrl/Shift), and keyboard navigation.
  - Interactive minimap for coarse navigation and markers that snap to peak samples.
- **Plugins & Automation**:
  - **Alias Sorter**: Organize your voicebank by alphabet, file, type, etc.
  - **Consistency Checker**: Detect and jump to errors like missing files or invalid timing.
  - **Duplicate Detector**: Find and prune exact or functional duplicates.
  - **Pitch Analyzer (Colheita)**: High-precision F0 detection (Hz + Musical Note) using parabolic interpolation.
  - **Enxertia (Batch Rename)**: Flexible search and replace for aliases with regex support.
  - **Batch Numeric Edit**: Apply values to multiple entries at once.
- **Workflow**:
  - **Undo/Redo**: Full history support with keyboard shortcuts (Ctrl+Z, Ctrl+Y).
  - **Live Preview**: Real-time parameter adjustments with live visual feedback.
  - **Playback**: Integrated rodio loop with real-time playhead and synthesis testing.
  - **Visual Customization**: Toggle spectrogram, grid, zero-line, and adjust colors/thickness.

## How to Run

1. Install Rust (https://rustup.rs/).
2. Clone the repository and run:
   ```bash
   cargo run
   ```

## Tech Stack

- **UI**: egui & eframe
- **Audio**: rodio & hound
- **DSP**: realfft
- **Language**: Rust
