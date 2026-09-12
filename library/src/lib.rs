#![forbid(unsafe_code)]

pub mod defaults {
    pub const TERRAIN_SCALE: f32 = 0.05;
    pub const MINERAL_THRESHOLD: f32 = 0.5;
    pub const TICK_DURATION: f32 = 1. / 10.;
    pub const STARTING_MINERALS: u32 = 100;
    pub const TERRAIN_SEED: u32 = 0;
    pub mod zone_extensions {
        pub const FACTORY: f32 = 60.;
    }
}

pub mod protocol {
    use serde::{Deserialize, Serialize};

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

        pub fn already_playing() -> Self {
            Self::new(false, "Already playing / name taken", (0., 0.))
        }

        pub fn invalid_name() -> Self {
            Self::new(false, "Invalid name", (0., 0.))
        }

        pub fn welcome(spawn_point: (f32, f32)) -> Self {
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
        AuthenticationResponse(AuthenticationResponse),
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

    pub fn is_name_valid(player_name: &str) -> bool {
        !player_name.is_empty()
    }
}
