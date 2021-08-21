use bevy::prelude::*;
use rand::prelude::random;

use std::{
    net::{TcpListener, TcpStream, IpAddr, Ipv4Addr, SocketAddr},
    u16,
};

use tungstenite::{
    accept_hdr,
    handshake::server::{Request, Response},
    protocol::Message,
    error::Error,
    WebSocket,
};

fn main() {
    App::build()
        .insert_resource(ClearColor(Color::BLACK))
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup", SystemStage::single(spawn_player.system()))
        .add_system(websocket_handshake.system())
        .add_system(websocket_server.system())
        .add_system(player_movement.system())
        .add_plugins(DefaultPlugins)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    
    const PORT: u16 = 3012;
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), PORT);
    let server = TcpListener::bind(addr).expect("Failed to bind server TcpListener.");
    server.set_nonblocking(true).expect("Failed to set server TcpListener to nonblocking.");
    commands.insert_resource(server);
    commands.insert_resource(Vec::<WebSocket<TcpStream>>::new());
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

fn websocket_handshake(
    server: Res<TcpListener>,
    mut ws: ResMut<Vec<WebSocket<TcpStream>>>,
) {
    for stream in server.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(_) => return,
        };

        let mut websocket = accept_hdr(stream, |req: &Request, response: Response| {
            info!("Received a new ws handshake at {}", req.uri());
            Ok(response)
        }).expect("Failed to accept websocket stream.");
        
        websocket
            .write_message(Message::text("Hello from Rust!"))
            .expect("Failed to write message.");
        
        ws.push(websocket);
    }
}

fn websocket_server(mut ws: ResMut<Vec<WebSocket<TcpStream>>>) {
    let mut closed_websockets = Vec::<usize>::new();
    for (i, websocket) in ws.iter_mut().enumerate() {
        let msg = match websocket.read_message() {
            Err(error) => {
                if let Error::ConnectionClosed = error {
                    info!("Connection closed");
                    closed_websockets.push(i);
                    continue;
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
            ).expect("Failed to write message.");
        }
    }
    for closed_websocket in closed_websockets {
        ws.remove(closed_websocket);
    }
}