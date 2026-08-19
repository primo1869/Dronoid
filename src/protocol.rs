use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum PlayerAction {
    Authentication { player_name: String },
    PlaceFactory { pos_x: f32, pos_y: f32, pos_z: f32 },
}

#[derive(Serialize, Deserialize)]
pub struct AuthenticationResponse {
    pub result: bool,
    pub text: String,
    pub spawn_point: (f32, f32),
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    AuthenticationResponse(AuthenticationResponse),
}
