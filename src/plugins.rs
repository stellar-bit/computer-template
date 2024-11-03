use std::cmp;
use std::fmt::Write;

mod build_spacecrafts;
// mod spacecraft_control;
mod spacecraft_construction;
mod spacecraft_controlv2;

pub use spacecraft_controlv2::{SpacecraftControl, SpacecraftState};
// pub use build_spacecrafts::BuildSpacecrafts;
pub use spacecraft_construction::SpacecraftConstruction;

use super::*;

use utils::{predictive_shoot_at, shoot_at, fly_to, improved_fly_to};

pub trait Plugin {
    fn update(&mut self, game_data: &mut GameData) {}
    fn update_ui(&mut self, ui: &mut egui::Ui) {}
    fn name(&self) -> String;
    fn update_interval(&mut self) -> &mut Interval;
}

