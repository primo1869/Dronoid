use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Bytes;

use crate::{Error, Result, game};

#[derive(Serialize, Deserialize)]
pub struct AuthenticationRequest {
    pub player_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthenticationResponse {
    pub result: bool,
    pub text: String,
    pub spawn_point: (f32, f32),
}

impl AuthenticationResponse {
    pub fn new(result: bool, text: &'static str, spawn_point: (f32, f32)) -> Self {
        Self {
            result,
            text: text.to_string(),
            spawn_point,
        }
    }

    pub(crate) fn already_playing() -> Self {
        Self::new(false, "Already playing / name taken", (0., 0.))
    }

    pub(crate) fn invalid_name() -> Self {
        Self::new(false, "Invalid name", (0., 0.))
    }

    pub(crate) fn welcome(spawn_point: (f32, f32)) -> Self {
        Self::new(true, "Welcome", spawn_point)
    }
}

#[derive(Serialize, Deserialize)]
pub enum FactoryOrder {
    SetAutoSpawn(bool),
    ManualSpawn,
}

#[derive(Serialize, Deserialize)]
pub struct FactoryControl {
    pub id: u32,
    pub order: FactoryOrder,
}

#[derive(Serialize, Deserialize)]
pub enum Action {
    PlaceFactory((f32, f32)),
    ControlFactory(FactoryControl),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Eq, Hash)]
pub enum Kind {
    Mineral,
    Factory,
    Spawn,
    Dronoid,
}

impl From<game::component::Kind> for Kind {
    fn from(value: game::component::Kind) -> Self {
        value.0
    }
}

#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub pos: (f32, f32),
    pub kind: Kind,
    pub id: u32,
}

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    pub entities_in_zone: Vec<Entity>,
    pub minerals_cnt: u32,
}

#[derive(Serialize, Deserialize)]
pub enum Response {
    PlaceFactory { result: bool },
    ControlFactory { result: bool },
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    AuthenticationRequest(AuthenticationRequest),
    Action(Action),
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    Response(Response),
    State(State),
}

pub fn is_name_valid(player_name: &String) -> bool {
    if player_name.is_empty() { false } else { true }
}

pub fn serialize<T: Serialize>(value: T) -> Result<Bytes> {
    Ok(Bytes::from(
        bson::serialize_to_vec(&value).map_err(|err| Error::Serde(err))?,
    ))
}

pub fn deserialize<T: for<'a> Deserialize<'a>>(bson: Bytes) -> crate::Result<T> {
    bson::deserialize_from_slice::<T>(bson.iter().as_slice()).map_err(|err| Error::Serde(err))
}
