use super::*;

#[derive(Debug)]
pub struct GameData<'a> {
    pub game: &'a mut Game, 
    pub player_id: PlayerId,
    pub network_game_cmds: &'a mut Vec<GameCmd>,
}

impl<'a> GameData<'a> {
    pub fn new(game: &'a mut Game, player_id: PlayerId, network_game_cmds: &'a mut Vec<GameCmd>) -> Self {
        Self {
            game,
            player_id,
            network_game_cmds,
        }
    }

    pub fn player(&self) -> &Player {
        self.game.players.get(&self.player_id).unwrap()
    }

    pub fn closest_my_star_base(&self, position: &Vec2) -> Option<(GameObjectId, StarBase)> {
        self.my_star_bases()
            .into_iter()
            .min_by(|(_, a), (_, b)| {
                a.body.position
                    .distance(*position)
                    .partial_cmp(&b.body.position.distance(*position))
                    .unwrap()
            })
    }

    pub fn my_star_bases(&self) -> BTreeMap<GameObjectId, StarBase> {
        self.game_objects
            .iter()
            .filter_map(|(id, game_object)| {
                if let GameObject::StarBase(star_base) = game_object {
                    if star_base.owner == self.player_id {
                        Some((*id, star_base.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn my_spacecrafts(&self) -> BTreeMap<GameObjectId, Spacecraft> {
        self.game_objects
            .iter()
            .filter_map(|(id, game_object)| {
                if let GameObject::Spacecraft(spacecraft) = game_object {
                    if spacecraft.owner == self.player_id {
                        Some((*id, spacecraft.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn closest_star_base(&self, position: &Vec2) -> Option<(GameObjectId, StarBase)> {
        self.game_objects
            .iter()
            .filter_map(|(id, game_object)| {
                if let GameObject::StarBase(star_base) = game_object {
                    Some((*id, star_base.clone()))
                } else {
                    None
                }
            })
            .min_by_key(|(_, star_base)| position.distance(star_base.body.position) as u32)
    }
    pub fn closest_asteroid(&self, position: &Vec2) -> Option<(GameObjectId, Asteroid)> {
        self.game_objects
            .iter()
            .filter_map(|(id, game_object)| {
                if let GameObject::Asteroid(asteroid) = game_object {
                    Some((*id, asteroid.clone()))
                } else {
                    None
                }
            })
            .min_by_key(|(_, asteroid)| position.distance(asteroid.body.position) as u32)
    }
    pub fn closest_enemy_spacecraft(
        &self,
        position: &Vec2,
    ) -> Option<(GameObjectId, Spacecraft)> {
        self.game_objects
            .iter()
            .filter_map(|(id, game_object)| {
                if let GameObject::Spacecraft(spacecraft) = game_object {
                    if spacecraft.owner != self.player_id {
                        Some((*id, spacecraft.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .min_by_key(|(_, spacecraft)| position.distance(spacecraft.body.position) as u32)
    }
    pub fn closest_enemy_star_base(
        &self,
        position: &Vec2,
    ) -> Option<(GameObjectId, StarBase)> {
        self.game_objects
            .iter()
            .filter_map(|(id, game_object)| {
                if let GameObject::StarBase(star_base) = game_object {
                    if star_base.owner != self.player_id {
                        Some((*id, star_base.clone()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .min_by_key(|(_, star_base)| position.distance(star_base.body.position) as u32)
    }

    pub fn closest_enemy_target(
        &self,
        position: &Vec2,
    ) -> Option<(GameObjectId, GameObjectBody)> {

        let closest_enemy_spacecraft = self
            .closest_enemy_spacecraft(&position)
            .map(|(id, spacecraft)| (id, spacecraft.body));
        let closest_asteroid = self
            .closest_asteroid(&position)
            .map(|(id, asteroid)| (id, asteroid.body));
        let closest_star_base = self
            .closest_enemy_star_base(&position)
            .map(|(id, star_base)| (id, star_base.body));

        let targets = vec![
            closest_enemy_spacecraft,
            closest_asteroid,
            closest_star_base,
        ];

        let targets = targets
            .into_iter()
            .filter_map(|target| target)
            .collect::<Vec<(GameObjectId, GameObjectBody)>>();

        targets
            .into_iter()
            .min_by_key(|(_, body)| position.distance(body.position) as u32)
    }

    pub fn execute_cmd(&mut self, cmd: GameCmd) {
        if let Err(err) = self.game.execute_cmd(User::Player(self.player_id), cmd.clone()) {
            println!("Error executing cmd: {:?}", err);
            return;
        }
        self.network_game_cmds.push(cmd);
    }

    pub fn execute_cmds(&mut self, cmds: Vec<GameCmd>) {
        for cmd in cmds {
            self.execute_cmd(cmd);
        }
    }
}

impl<'a> std::ops::Deref for GameData<'a> {
    type Target = Game;

    fn deref(&self) -> &Self::Target {
        &self.game
    }
}

impl<'a> std::ops::DerefMut for GameData<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.game
    }
}
