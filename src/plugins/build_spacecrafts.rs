use super::*;


#[derive(Serialize, Deserialize)]
pub struct BuildSpacecrafts {
    structure: SpacecraftStructure,
    tag: String,
    max_parallel: usize,
    max_total: Option<usize>,
    pub interval: Interval
}

impl BuildSpacecrafts {
    pub fn new(
        structure: SpacecraftStructure,
        tag: String,
        max_parallel: usize,
        max_total: Option<usize>,
    ) -> Self {
        if !structure.valid() {
            panic!("structure {:?} non valid, quitting", structure);
        }
        Self {
            structure,
            tag,
            max_parallel,
            max_total,
            interval: Interval::new(time::Duration::from_secs(2))
        }
    }
   
}

impl Plugin for BuildSpacecrafts {

    fn name(&self) -> String {
        format!("Build {}", self.tag)
    }
    fn update(&mut self, game_data: &mut GameData) {
        let star_bases = game_data.my_star_bases();
        let mut remaining_parallel = self.max_parallel;

        let mut hangars_available = vec![];

        let mut total = 0;
        for (id, spacecraft) in game_data.my_spacecrafts() {
            if spacecraft.tags.contains(&self.tag.to_string()) {
                total += 1;
            }
        }

        if let Some(max_total) = self.max_total {
            if total >= max_total {
                return;
            }
        }


        for (id, star_base) in &star_bases {
            for (index, hangar) in star_base.hangars.iter().enumerate() {
                for spacecraft_structure in &hangar.building_queue {
                    if spacecraft_structure.tags.contains(&self.tag.to_string()) && remaining_parallel > 0 {
                        if remaining_parallel > 0 {
                            remaining_parallel -= 1;
                        }
                        if hangar.build_finished() {
                            game_data.execute_cmd(GameCmd::DeploySpacecraft(*id, index));
                        }
                    }
                }
                if hangar.building_queue.is_empty() {
                    hangars_available.push((*id, index, hangar));
                }
            }
        }

        for (star_base_id, hangar_index, _) in hangars_available.iter().take(remaining_parallel) {
            if game_data.player().has_materials(&self.structure.materials()) {
                game_data.execute_cmd(GameCmd::BuildSpacecraft(
                    *star_base_id,
                    self.structure.clone(),
                    *hangar_index,
                ));
            }
        }
    }
    fn update_ui(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Max parallel: {}", self.max_parallel));
        ui.label(format!("Max total: {:?}", self.max_total));
        ui.label(format!("Tag: {}", self.tag));
        ui.label(format!("Structure: {:?}", self.structure));
    }

    fn update_interval(&mut self) -> &mut Interval {
        &mut self.interval 
    }
}