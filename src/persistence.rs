use crate::game::helper::gen_id;

#[derive(Clone)]
pub struct PlayerEntry {
    pub(crate) id: u32,
    pub(crate) name: String,
    pub(crate) spawn_point: (f32, f32),
}

#[derive(Default)]
pub struct Database {
    players: Vec<PlayerEntry>,
}

impl Database {
    pub async fn get_player_entry(&self, name: &String) -> Option<PlayerEntry> {
        if let Some(player_entry) = self.players.iter().find(|x| x.name == *name) {
            Some(player_entry.clone())
        } else {
            None
        }
    }

    pub async fn add_player_entry(&mut self, name: String, spawn_point: (f32, f32)) -> PlayerEntry {
        let player_entry = PlayerEntry {
            id: gen_id(),
            spawn_point,
            name,
        };
        self.players.push(player_entry.clone());
        player_entry
    }
}
