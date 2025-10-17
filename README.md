# Face Tracking Eyes

A Rust application that combines real-time face detection with 3D animated eyes that follow detected faces. This project merges the face recognition capabilities from a webcam with beautifully rendered 3D eyes using the Bevy game engine.

## Features

- 🎥 **Real-time Face Detection**: Uses your webcam to detect faces in real-time
- 👁️ **3D Eye Rendering**: Beautiful 3D eye model that smoothly tracks faces
- 🎯 **Natural Movement**: Smooth interpolation for realistic eye movement
- ⚡ **Efficient**: Runs face detection in a separate thread for optimal performance

## How It Works

1. The application starts a background thread that captures video from your webcam
2. Face detection (using the SeetaFace model) identifies faces in each frame
3. Face positions are normalized and shared with the main rendering thread
4. The 3D eyes smoothly rotate to look at detected faces
5. If no face is detected, the eyes return to center

## Requirements

- Rust 1.70 or later
- A webcam
- macOS, Linux, or Windows

## Installation

1. Clone or navigate to this directory:
```bash
cd face-tracking-eyes
```

2. Build the project (this will download the face detection model if needed):
```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

Or run the compiled binary directly:
```bash
./target/release/face-tracking-eyes
```

## Controls

- **ESC** or **Q**: Quit the application

## Technical Details

### Dependencies

- **bevy**: 3D rendering and game engine
- **nokhwa**: Cross-platform webcam access
- **rustface**: Face detection using the SeetaFace algorithm
- **image**: Image processing

### Architecture

The application uses a multi-threaded architecture:
- **Main Thread**: Runs the Bevy app, rendering the 3D eyes
- **Camera Thread**: Captures frames, performs face detection, and shares results

Face positions are shared between threads using an `Arc<Mutex<Option<(f32, f32)>>>`, providing thread-safe access to the latest detected face position.

### Face Detection Model

The application uses the SeetaFace frontal face detection model (`seeta_fd_frontal_v1.0.bin`). If the model file is not present, it will be automatically downloaded on first run.

## Credits

This project combines code from:
- Original "eyes" project (3D eye rendering with Bevy)
- Original "face_recognition" project (webcam face detection)

## License

See individual component licenses for details.

