use bevy::prelude::*;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;
use rustface::ImageData;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Component)]
struct Eye;

#[derive(Resource, Clone)]
struct FacePosition {
    position: Arc<Mutex<Option<(f32, f32)>>>, // Normalized position (-1 to 1)
}

fn main() {
    println!("Starting face-tracking eyes...");
    
    // Initialize face position resource
    let face_position = FacePosition {
        position: Arc::new(Mutex::new(None)),
    };
    
    // Clone for the camera thread
    let face_position_clone = face_position.clone();
    
    // Start face detection in a separate thread
    thread::spawn(move || {
        if let Err(e) = run_face_detection(face_position_clone) {
            eprintln!("Face detection error: {}", e);
        }
    });
    
    // Start the Bevy app with eye rendering
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Face Tracking Eyes - Press ESC to quit".to_string(),
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(face_position)
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, eye_follow_face))
        .run();
}

fn run_face_detection(face_position: FacePosition) -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing camera...");
    
    // Set up camera
    let camera_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let mut camera = Camera::new(CameraIndex::Index(0), camera_format)?;
    camera.open_stream()?;
    println!("Camera opened successfully");
    
    // Initialize face detector
    println!("Initializing face detector...");
    let model_path = "seeta_fd_frontal_v1.0.bin";
    
    // Download model if it doesn't exist
    if !std::path::Path::new(model_path).exists() {
        println!("Downloading face detection model...");
        let model_url = "https://raw.githubusercontent.com/atomashpolskiy/rustface/master/model/seeta_fd_frontal_v1.0.bin";
        let response = std::process::Command::new("curl")
            .args(&["-L", "-o", model_path, model_url])
            .output()
            .expect("Failed to download model");
        
        if !response.status.success() {
            eprintln!("Failed to download face detection model");
            eprintln!("Please download manually from: {}", model_url);
            return Ok(());
        }
        println!("Model downloaded successfully");
    }
    
    let mut detector = rustface::create_detector(model_path)?;
    detector.set_min_face_size(30);
    detector.set_score_thresh(1.0);
    detector.set_pyramid_scale_factor(0.8);
    detector.set_slide_window_step(4, 4);
    
    println!("Face detector initialized - eyes will track detected faces");
    
    let mut frame_count = 0;
    
    loop {
        // Capture frame from camera
        let frame = match camera.frame() {
            Ok(frame) => frame,
            Err(e) => {
                eprintln!("Error capturing frame: {}", e);
                thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
        };
        
        frame_count += 1;
        
        // Decode the frame
        let decoded = match frame.decode_image::<RgbFormat>() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error decoding frame: {}", e);
                continue;
            }
        };
        
        let width = decoded.width() as usize;
        let height = decoded.height() as usize;
        
        // Convert to grayscale for face detection
        let rgb_data = decoded.to_vec();
        let gray_data = create_gray_data(&rgb_data, width, height);
        let gray_image = ImageData::new(&gray_data, width as u32, height as u32);
        
        // Detect faces
        let faces = detector.detect(&gray_image);
        
        // Update face position if faces detected
        if let Some(face) = faces.first() {
            let bbox = face.bbox();
            
            // Calculate center of the face
            let center_x = bbox.x() + bbox.width() as i32 / 2;
            let center_y = bbox.y() + bbox.height() as i32 / 2;
            
            // Normalize to -1 to 1 range
            let norm_x = (center_x as f32 / width as f32) * 2.0 - 1.0;
            let norm_y = -((center_y as f32 / height as f32) * 2.0 - 1.0); // Flip Y
            
            if let Ok(mut pos) = face_position.position.lock() {
                *pos = Some((norm_x, norm_y));
            }
            
            if frame_count % 60 == 0 {
                println!("Tracking face at ({:.2}, {:.2})", norm_x, norm_y);
            }
        } else {
            // No face detected - clear position
            if let Ok(mut pos) = face_position.position.lock() {
                *pos = None;
            }
        }
        
        // Limit to reasonable frame rate
        thread::sleep(std::time::Duration::from_millis(33)); // ~30 FPS
    }
}

fn create_gray_data(rgb_data: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut gray = Vec::with_capacity(width * height);
    
    for i in 0..(width * height) {
        let idx = i * 3;
        let r = rgb_data[idx] as f32;
        let g = rgb_data[idx + 1] as f32;
        let b = rgb_data[idx + 2] as f32;
        
        // Convert to grayscale using luminance formula
        let gray_val = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
        gray.push(gray_val);
    }
    
    gray
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load and spawn the eye model
    commands.spawn((
        SceneBundle {
            scene: asset_server.load("eye-model/source/eye-model.gltf#Scene0"),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Eye,
    ));
    
    // Camera positioned to look from the right side (+X axis)
    let camera_pos = Vec3::new(5.0, 0.0, 0.0);
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(camera_pos.x, camera_pos.y, camera_pos.z)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    
    // Light positioned behind the camera
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 2000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(6.0, 1.0, 0.0),
        ..default()
    });
    
    // Add ambient light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 3000.0,
            ..default()
        },
        transform: Transform::from_xyz(5.0, 3.0, 2.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    
    println!("3D eye rendering initialized");
    println!("Eyes will track any detected faces from your webcam");
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<bevy::app::AppExit>,
) {
    // Press ESC or Q to quit
    if keyboard_input.just_pressed(KeyCode::Escape) || keyboard_input.just_pressed(KeyCode::KeyQ) {
        exit.send(bevy::app::AppExit::Success);
    }
}

fn eye_follow_face(
    mut eye_query: Query<&mut Transform, With<Eye>>,
    face_position: Res<FacePosition>,
) {
    // Get the current face position
    let face_pos = if let Ok(pos) = face_position.position.lock() {
        *pos
    } else {
        None
    };
    
    // If no face detected, return to center
    let (norm_x, norm_y) = face_pos.unwrap_or((0.0, 0.0));
    
    // Define eye's natural range of motion in radians
    let max_yaw = std::f32::consts::PI / 4.0;   // ±45 degrees horizontal
    let max_pitch = std::f32::consts::PI / 6.0; // ±30 degrees vertical
    
    // Map face position (-1 to 1) directly to eye rotation angles
    // norm_x/norm_y of -1 = bottom/left of frame, +1 = top/right of frame
    let target_yaw = -norm_x * max_yaw;      // Negative to mirror camera view
    let target_pitch = norm_y * max_pitch;   // Direct mapping
    
    // Rotate eye to the target angles
    for mut transform in eye_query.iter_mut() {
        // Create rotation from yaw (left/right) and pitch (up/down)
        let target_rotation = Quat::from_rotation_y(target_yaw) * Quat::from_rotation_z(target_pitch);
        
        // Smooth interpolation for natural movement
        transform.rotation = transform.rotation.slerp(target_rotation, 0.15);
    }
}
