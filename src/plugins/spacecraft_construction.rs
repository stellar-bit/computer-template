use std::path::PathBuf;

use super::*;

#[derive(Serialize)]
pub struct SpacecraftConstruction {
    spacecraft_structures_path: PathBuf,
    update_interval: Interval,
    auto_deploy: bool,
    spacecraft_structures: Vec<(String, SpacecraftStructure)>,
    build_queue: Vec<SpacecraftStructure>
}

impl SpacecraftConstruction {
    pub fn new(spacecraft_structures_path: PathBuf) -> Self {
        Self {
            spacecraft_structures_path,
            update_interval: Interval::new(time::Duration::from_millis(500)),
            auto_deploy: true,
            spacecraft_structures: vec![],
            build_queue: vec![]
        }
    }
}

impl Plugin for SpacecraftConstruction {
    fn name(&self) -> String {
        "spacecraft construction".into()
    }

    fn update_interval(&mut self) -> &mut Interval {
        &mut self.update_interval
    }

    fn update(&mut self, game_data: &mut GameData) {
        if self.auto_deploy {
            for (star_base_id, star_base) in game_data.my_star_bases() {
                for (hangar_index, hangar) in star_base.hangars.into_iter().enumerate() {
                    if hangar.build_finished() {
                        game_data.execute_cmd(GameCmd::DeploySpacecraft(star_base_id, hangar_index))
                    }
                }
            } 
        }
        self.spacecraft_structures = vec![];
        for dir_entry in self.spacecraft_structures_path.read_dir().unwrap() {
            let dir_entry = dir_entry.unwrap(); 
            let structure_path = dir_entry.path();
            assert!(structure_path.extension().unwrap() == "json");
            let structure_data_raw = std::fs::read_to_string(structure_path.clone()).unwrap();
            let structure_data = deserialize_str(&structure_data_raw).unwrap();
            self.spacecraft_structures.push((structure_path.file_name().unwrap().to_str().unwrap().into(), structure_data));
        }

        for spacecraft_structure in std::mem::take(&mut self.build_queue) {
            let mut hangars = game_data.my_star_bases().into_iter().map(|(id, star_base)| star_base.hangars.into_iter().enumerate().map(|(index, hangar)| {
                let total_build_time = hangar.building_queue.iter().map(|structure| structure.build_time()).sum();
                (total_build_time-hangar.progress.min(total_build_time), id, index)
            }).collect::<Vec<_>>()).flatten().collect::<Vec<_>>();

            hangars.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            if let Some((_, starbase_id, hangar_index)) = hangars.get(0) {
                game_data.execute_cmd(GameCmd::BuildSpacecraft(*starbase_id, spacecraft_structure, *hangar_index));
            }
        }
    }

    fn update_ui(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.auto_deploy, "auto deploy");
        
        for (name, spacecraft_structure) in &self.spacecraft_structures {
            if ui.button(name).clicked() {
                self.build_queue.push(spacecraft_structure.clone());
            }
        }
    }
}