use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum FactoryControl {
    SetAutoSpawn(bool),
    ManualSpawn,
}

#[derive(Serialize, Deserialize)]
pub enum PlayerAction {
    Authentication { player_name: String },
    PlaceFactory((f32, f32)),
    ControlFactory { id: i64, control: FactoryControl },
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
