# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added
- **Plugin: Pitch Analyzer (Colheita)**: Visualizes the pitch trajectory in Hz over time, with reference lines for musical notes (C, D, E...).
- **Plugin: Duplicate Detector (Podador)**: Finds exact, case-insensitive, or functional duplicates (same audio slice). Allows jumping to or deleting redundant entries.
- **Multi-Selection support**: Select multiple rows using **Ctrl + Click** or **Shift + Click / Shift + Arrow Keys**.
- **Plugin: Consistency Checker (Inspetor)**: Analyzes the whole voicebank for errors like:
  - Missing audio files.
  - Empty aliases.
  - Negative offsets, preutterance, or consonant.
  - Logical errors where Overlap > Preutterance.
- **Plugin: Alias Sorter**: Organize entries by alphabet, reversa, filename, type, length, or offset.
  - Supports "Group by file" and "Completed first" options.
- **Waveform Customization**: New settings panel to change positive/negative colors, thickness, and render modes (Spline / Blocks).
- **Spectrogram Customization**: Configure FFT Size (+4096), Hop Size, Gamma, Colormaps (Fire, Inferno, etc.), and Frequency Range.
- **Synthesis Test**: Integration for resampling voices within the app.

### Fixed
- **Major Bug: Undo/Redo 'Maintained' plugin edits**: Fixed logic where focused widgets or active drags would overwrite restored data. Now resets focus and drag states upon Undo/Redo.
- **Spectrogram/Waveform Alignment**: Corrected the fft_size / 2 offset and floating-point precision issues to ensure visually perfect alignment.
- **Spectrogram Quality**: Upgraded from bilinear to **Bicubic Interpolation (Catmull-Rom)** to prevent pixelation during high zoom levels.
- **Waveform Fidelity**: Implemented continuous spline rendering for zoomed-in waves and accurate peak envelopes for zoomed-out views.

### Refactored
- Separated plugin logic into src/plugins.rs.
- Decoupled spectrogram recomputation from parameter updates to avoid lag.
- Moved spectrogram data management to the main app loop for better settings access.
- Corrected the deep undo stack logic to prevent skipping states.
- Cleaned up compiler warnings related to unused variables and field reads.
