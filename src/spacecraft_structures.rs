use super::*;

pub fn asteroid_miner() -> SpacecraftStructure {
    SpacecraftStructure {
        component_placeholders: vec![
            ComponentPlaceholder::new(ComponentType::Central, ivec2(0, 0), Orientation::Up),
            ComponentPlaceholder::new(ComponentType::SteelBlock, ivec2(0, 1), Orientation::Up),
            ComponentPlaceholder::new(ComponentType::SteelBlock, ivec2(0, -1), Orientation::Up),
            ComponentPlaceholder::new(ComponentType::SteelBlock, ivec2(1, 0), Orientation::Up),
            ComponentPlaceholder::new(ComponentType::SteelBlock, ivec2(-1, 0), Orientation::Up),
            ComponentPlaceholder::new(ComponentType::RaptorEngine, ivec2(-1, -2), Orientation::Up),
            ComponentPlaceholder::new(ComponentType::RaptorEngine, ivec2(1, -2), Orientation::Up),
            ComponentPlaceholder::new(ComponentType::LaserWeapon, ivec2(0, 1), Orientation::Up),
            ComponentPlaceholder::new(ComponentType::LaserWeapon, ivec2(0, -1), Orientation::Up),
            ComponentPlaceholder::new(ComponentType::LaserWeapon, ivec2(1, 0), Orientation::Up),
            ComponentPlaceholder::new(ComponentType::LaserWeapon, ivec2(-1, 0), Orientation::Up),
        ],
        tags: vec!["asteroid_miner".to_string()],
    }
}