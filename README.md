# egui_plot

[<img alt="github" src="https://img.shields.io/badge/github-emilk/egui_plot-8da0cb?logo=github" height="20">](https://github.com/emilk/egui_plot)
[![Latest version](https://img.shields.io/crates/v/egui_plot.svg)](https://crates.io/crates/egui_plot)
[![Documentation](https://docs.rs/egui_plot/badge.svg)](https://docs.rs/egui_plot)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)
[![Discord](https://img.shields.io/discord/900275882684477440?label=egui%20discord)](https://discord.gg/JFcEma9bJq)

Immediate mode plotting for [`egui`](https://github.com/emilk/egui).

![egui_plot_white](https://github.com/user-attachments/assets/b29acf5e-ccbf-4cb7-b03b-7e258fa5db16)

[Try the web demo](https://emilk.github.io/egui_plot/)

### Testing
* Locally: `cargo run -p demo`
* Web: `(cd demo && trunk serve)`


### History
This crate was originally hosted at https://github.com/emilk/egui but was extracted into its own repository on 2024-07-15.

## MDF Integration TODO List

### 1. Dependency Setup ✅
- [x] Add rsmdf dependency to `demo/Cargo.toml`
- [x] Configure the dependency to use your GitHub fork
- [x] Add any necessary additional dependencies for file handling

### 2. MDF File Loading ✅
- [x] Create a new struct to manage MDF file state
- [x] Implement file selection dialog using egui's file picker
- [x] Add error handling for file loading

### 3. Signal Tree UI
- [x] Create a new panel component for signal selection
- [x] Implement tree view of MDF channels using egui's tree widget
- [x] Add checkboxes for signal selection
- [x] Add search/filter functionality for signals
- [x] Add signal grouping by data group/channel group
- [ ] Add signal metadata display (units, sampling rate, etc.)

### 4. Plot Integration
- [ ] Create a signal data structure to hold selected signals
- [ ] Implement signal data loading from MDF
- [ ] Add real-time plot updates when signals are selected/deselected
- [ ] Add time axis synchronization across plots
- [ ] Add plot controls (zoom, pan, etc.)
- [ ] Add signal legend with color coding

### 5. Performance Optimization
- [ ] Implement lazy loading of signal data
- [ ] Add data caching for frequently accessed signals
- [ ] Optimize plot rendering for large datasets
- [ ] Add progress indicators for long operations

### 6. UI/UX Improvements
- [ ] Add keyboard shortcuts for common operations
- [ ] Add drag-and-drop support for signal reordering
- [ ] Add signal comparison tools
- [ ] Add export functionality for selected signals
- [ ] Add configuration persistence

### 7. Testing & Documentation
- [x] Add unit tests for MDF parsing
- [x] Add integration tests for UI components
- [ ] Add documentation for new features
- [ ] Add example usage
