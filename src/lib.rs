#![feature(let_chains)]

use stellar_bit_core::prelude::*;
use serde::{Serialize, Deserialize};

mod game_data;
use game_data::GameData;

mod plugins;
use plugins::{Plugin,SpacecraftControl};

mod utils;

mod spacecraft_structures;

pub static mut STARTUP: bool = true;
pub static mut PLUGIN_MANAGER: Option<PluginManager> = None;

#[derive(Default)]
pub struct PluginManager {
    plugins: Vec<(bool, Box<dyn Plugin>)>
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
pub extern fn execute(pointers: *const (*mut Game, *const User, *const egui::Context, *mut Vec<GameCmd>)) {
    use std::path::Path;

    use plugins::SpacecraftConstruction;

    let (game_ptr, user_ptr, egui_ctx_ptr, network_game_cmds_ptr) = unsafe { *pointers };

    let game = unsafe { &mut *game_ptr };
    let user = unsafe { &*user_ptr };
    let egui_ctx = unsafe { &*egui_ctx_ptr };
    let network_game_cmds = unsafe { &mut *network_game_cmds_ptr };


    let mut plugin_manager = unsafe { PLUGIN_MANAGER.take().unwrap_or_default() };
    
    let startup = unsafe { &mut STARTUP };
    if *startup {
        plugin_manager.plugins = vec![];
        plugin_manager.plugins.push(
            (false, Box::new(SpacecraftControl::new()))
        );
        plugin_manager.plugins.push(
            (false, Box::new(SpacecraftConstruction::new(Path::new("./spacecraft-structures").into())))
        );
        *startup = false;
    }

    let User::Player(player_id) = user else {
        return;
    };

    let mut game_data = GameData::new(game, *player_id, network_game_cmds);

    egui::SidePanel::new(egui::panel::Side::Right, "Plugins").show(egui_ctx, |ui| {
        for (enabled, plugin) in plugin_manager.plugins.iter_mut() {
            ui.checkbox(enabled, plugin.name());
        }
    });

    for (enabled, plugin) in plugin_manager.plugins.iter_mut() {
        if !*enabled {
            continue;
        }
        egui::Window::new(plugin.name()).show(egui_ctx, |ui| {
            plugin.update_ui(ui);
            if plugin.update_interval().check() {
                plugin.update(&mut game_data);
            }
        });
    }

    unsafe { PLUGIN_MANAGER = Some(plugin_manager) };
}

// fn get_plugin_manager() -> anyhow::Result<PluginManager> {
//     let plugin_manager_raw = std::fs::read("plugin_manager.bin")?;
//     let plugin_manager = deserialize_bytes(&plugin_manager_raw)?;

//     Ok(plugin_manager)
// }

// fn save_plugin_manager(plugin_manager: &PluginManager) {
//     let plugin_manager_raw = serialize_bytes(plugin_manager).unwrap();
//     std::fs::write("plugin_manager.bin", plugin_manager_raw).unwrap();
// }




#[cfg(target_arch = "wasm32")]
pub static mut PLUGIN_MANAGER: Option<PluginManager> = None;

pub static mut UPDATE_TOGGLE: bool = false;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn execute(game_bin: Vec<u8>, user_bin: Vec<u8>) -> Vec<u8> {
    let mut game: Game = deserialize_bytes(&game_bin).unwrap();
    let user: User = deserialize_bytes(&user_bin).unwrap();

    let User::Player(player_id) = user else {
        return vec![];
    };

    let mut result = vec![];

    let mut game_data = GameData::new(&mut game, player_id, &mut result);

    if unsafe {UPDATE_TOGGLE} {
        unsafe {UPDATE_TOGGLE = !UPDATE_TOGGLE};
        unsafe {PLUGIN_MANAGER = None};
    }

    let mut plugin_manager = (unsafe { PLUGIN_MANAGER.take() }).unwrap_or({
        let mut result = PluginManager::default();
        result.plugins.push(
            (true, Plugin::SpacecraftControl(SpacecraftControl::new("asteroid_miner".into(), plugins::SpacecraftState::Idle)))
        );
        result.plugins.push(
            (true, Plugin::BuildSpacecrafts(BuildSpacecrafts::new(spacecraft_structures::asteroid_miner(), "asteroid_miner".into(), 2, None)))
        );
        result
    });


    for (enabled, plugin) in plugin_manager.plugins.iter_mut() {
        if !*enabled {
            continue;
        }
        if plugin.interval().check() {
            plugin.update(&mut game_data);
        }
    }

    unsafe { PLUGIN_MANAGER = Some(plugin_manager) };
    
    serialize_bytes(&result).unwrap()
}