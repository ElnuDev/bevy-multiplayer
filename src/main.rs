use bevy::prelude::*;
use rand::prelude::random;

use std::{
    net::TcpListener,
    thread::spawn,
};

use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
    protocol::Message,
    error::Error,
};

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup", SystemStage::single(spawn_player.system()))
        .add_system(player_movement.system())
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    
    // Start websocket server on separate thread to prevent blocking
    spawn(|| handshake());
}

fn spawn_player(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::hsl(random::<f32>() * 255.0, 0.75, 0.75).into()),
            sprite: Sprite::new(Vec2::new(32.0, 32.0)),
            ..Default::default()
        })
        .insert(Player);
}


fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_transforms: Query<&mut Transform, With<Player>>
) {
    for mut player_transform in player_transforms.iter_mut() {
        let mut delta_x: f32 = 0.0;
        let mut delta_y: f32 = 0.0;
        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            delta_x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            delta_x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            delta_y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            delta_y += 1.0;
        }
        
        if delta_x == 0.0 && delta_y == 0.0 {
            continue;
        }

        // Get input length
        let delta_length = (delta_x * delta_x + delta_y * delta_y).sqrt();
        
        const SPEED: f32 = 5.0;

        if delta_x != 0.0 {
            // Normalize
            delta_x /= delta_length;

            delta_x *= SPEED;

            player_transform.translation.x += delta_x;
        }

        if delta_y != 0.0 {
            // Normalize
            delta_y /= delta_length;

            delta_y *= SPEED;
            
            player_transform.translation.y += delta_y;
        }
    }
}

struct Player;

fn handshake() {
    let server = TcpListener::bind("127.0.0.1:3012").unwrap();
    for stream in server.incoming() {
        spawn(move || {
            let callback = |req: &Request, response: Response| {
                println!("Received a new ws handshake at {}", req.uri());
                Ok(response)
            };
            let mut websocket = accept_hdr(stream.unwrap(), callback).unwrap();
            
            websocket.write_message(Message::text("Hello from Rust!")).unwrap();

            loop {
                let msg = match websocket.read_message() {
                    Err(error) => {
                        if let Error::ConnectionClosed = error {
                            println!("Connection closed");
                            break;
                        }
                        panic!("{}", error);
                    },
                    Ok(message) => message,
                };
                if msg.is_binary() || msg.is_text() {
                    websocket.write_message(
                        Message::text(
                            format!("Your message was {} bytes long!", msg.to_text().unwrap().len())
                        )
                    ).unwrap();
                }
            }
        });
    }
}