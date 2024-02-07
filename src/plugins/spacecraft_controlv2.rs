
use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone, Copy)]
pub enum SpacecraftState {
    #[default]
    Idle,
    Attack,
    Mining,
    Defense
}

#[derive(Serialize, Deserialize)]
pub struct SpacecraftControl {
    spacecraft_states: HashMap<GameObjectId, SpacecraftState>,
    interval: Interval,
    selectable_tags: Vec<(String, bool)>,
    selectable_state: SpacecraftState,
    new_tag_input: String,
    spacecraft_tags: HashMap<GameObjectId, Vec<String>>
}

impl SpacecraftControl {
    pub fn new() -> Self {
        Self {
            spacecraft_states: HashMap::new(),
            interval: Interval::new(time::Duration::from_millis(300)),
            selectable_tags: vec![],
            selectable_state: Default::default(),
            new_tag_input: String::new(),
            spacecraft_tags: Default::default()
        }
    }
}

impl Plugin for SpacecraftControl {
    fn name(&self) -> String {
        format!("spacecraft control")
    }

    fn update_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.new_tag_input);
            if ui.button("Add tag").clicked() {
                self.selectable_tags.push((std::mem::take(&mut self.new_tag_input), false));
            }
        });
        for (tag_name, selected) in &mut self.selectable_tags {
            ui.checkbox(selected, tag_name.clone());
        }

        egui::ComboBox::from_label("State")
            .selected_text(
                format!("{:?}", self.selectable_state)
            )
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.selectable_state, SpacecraftState::Idle, "Idle");
                ui.selectable_value(&mut self.selectable_state, SpacecraftState::Attack, "Attack");
                ui.selectable_value(&mut self.selectable_state, SpacecraftState::Mining, "Mining");
                ui.selectable_value(&mut self.selectable_state, SpacecraftState::Defense, "Defense");
            });

        if ui.button("Apply").clicked() {
            let active_tags = self.selectable_tags.clone().into_iter().filter_map(|x| if x.1 {Some(x.0)} else {None}).collect::<Vec<_>>();
            for (id, tags) in &self.spacecraft_tags {
                for active_tag in &active_tags {
                    if tags.contains(active_tag) {
                        *self.spacecraft_states.entry(*id).or_default() = self.selectable_state;
                    }
                }
            }
        }

        ui.collapsing("Spacecraft states", |ui| {
            for (id, spacecraft_state) in &self.spacecraft_states {
                ui.label(format!("{}: {:?}", id, spacecraft_state));
            }
        });
    }
    
    fn update(&mut self, game_data: &mut GameData) {
        let spacecrafts = game_data.my_spacecrafts();

        self.spacecraft_states.retain(|id, _| {
            spacecrafts.contains_key(id)
        });

        self.spacecraft_tags.retain(|id, _| {
            spacecrafts.contains_key(id)
        });

        for (id, spacecraft) in &spacecrafts {
            let spacecraft_state = *self.spacecraft_states.entry(*id).or_default();
            self.spacecraft_tags.insert(*id, spacecraft.tags.clone());
            match spacecraft_state {
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
                        game_data.execute_cmds(impulse_fly_to((id, spacecraft), enemy_starbase.1.body, 0.7));
                    }
                }
                SpacecraftState::Mining => {
                    let least_material = *game_data.player().materials.iter().min_by(|&a, &b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
                    let closest_asteroid_with_least_material = game_data.asteroids().into_iter().filter(|asteroid| asteroid.material == least_material).min_by(|&a, &b| a.body.position.distance(spacecraft.body.position).partial_cmp(&b.body.position.distance(spacecraft.body.position)).unwrap());

                    let mut has_taken_shot = false;
                    if let Some(asteroid) = closest_asteroid_with_least_material.cloned() {
                        game_data.execute_cmds(impulse_fly_to((id, spacecraft), asteroid.body.clone(), 0.6));
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
                        game_data.execute_cmds(impulse_fly_to((id, spacecraft), star_base.1.body, 0.6));
                    }
                }
            }
        }

    }
    fn update_interval(&mut self) -> &mut Interval {
        &mut self.interval
    }
}


