
use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum SpacecraftState {
    #[default]
    Idle,
    Attack,
    Mining,
    Defense
}

#[derive(Serialize, Deserialize)]
pub struct SpacecraftControl {
    pub tag: String,
    pub state: SpacecraftState,
    interval: Interval
}

impl SpacecraftControl {
    pub fn new(tag: String, state: SpacecraftState) -> Self {
        Self {
            tag,
            interval: Interval::new(time::Duration::from_millis(300)),
            state,
        }
    }
}

impl Plugin for SpacecraftControl {
    fn name(&self) -> String {
        format!("Control of {}", self.tag)
    }

    fn update_ui(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_label("State")
            .selected_text(
                format!("{:?}", self.state)
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.state, SpacecraftState::Idle, "Idle");
                ui.selectable_value(&mut self.state, SpacecraftState::Attack, "Attack");
                ui.selectable_value(&mut self.state, SpacecraftState::Mining, "Mining");
                ui.selectable_value(&mut self.state, SpacecraftState::Defense, "Defense");
            });
    }
    
    fn update(&mut self, game_data: &mut GameData) {
        let mut spacecrafts = game_data.my_spacecrafts();
        spacecrafts.retain(|_, spacecraft| spacecraft.tags.contains(&self.tag.to_string()));

        for (id, spacecraft) in &spacecrafts {
            match self.state {
                SpacecraftState::Idle => {
                    if let Some(target) = game_data.closest_enemy_target(&spacecraft.body.position) {
                        game_data.execute_cmds(predictive_shoot_at((id, spacecraft), target.1));
                    }
                }
                SpacecraftState::Attack => {
                    if let Some(target) = game_data.closest_enemy_target(&spacecraft.body.position) {
                        game_data.execute_cmds(predictive_shoot_at((id, spacecraft), target.1));
                    }
                    if let Some(enemy_starbase) = game_data.closest_enemy_star_base(&spacecraft.body.position) {
                        game_data.execute_cmds(improved_fly_to((id, spacecraft), enemy_starbase.1.body, 0.7));
                    }
                }
                SpacecraftState::Mining => {
                    let least_material = *game_data.player().materials.iter().min_by(|&a, &b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
                    let closest_asteroid_with_least_material = game_data.asteroids().into_iter().filter(|asteroid| asteroid.material == least_material).min_by(|&a, &b| a.body.position.distance(spacecraft.body.position).partial_cmp(&b.body.position.distance(spacecraft.body.position)).unwrap());

                    let mut has_taken_shot = false;
                    if let Some(asteroid) = closest_asteroid_with_least_material.cloned() {
                        game_data.execute_cmds(improved_fly_to((id, spacecraft), asteroid.body.clone(), 0.6));
                        if asteroid.body.position.distance(spacecraft.body.position) < 200.0 {
                            game_data.execute_cmds(predictive_shoot_at((id, spacecraft), asteroid.body));
                            has_taken_shot = true;
                        }
                    }

                    if !has_taken_shot && let Some(target) = game_data.closest_enemy_target(&spacecraft.body.position) {
                        game_data.execute_cmds(predictive_shoot_at((id, spacecraft), target.1));
                    }

                }
                SpacecraftState::Defense => {
                    if let Some(target) = game_data.closest_enemy_target(&spacecraft.body.position) {
                        game_data.execute_cmds(predictive_shoot_at((id, spacecraft), target.1));
                    }

                    if let Some(star_base) = game_data.closest_my_star_base(&spacecraft.body.position) {
                        game_data.execute_cmds(improved_fly_to((id, spacecraft), star_base.1.body, 0.6));
                    }
                }
            }
        }
    }
    fn update_interval(&mut self) -> &mut Interval {
        &mut self.interval
    }
}


