use super::*;

pub fn shoot_at(
    (spacecraft_id, spacecraft): (&GameObjectId, &Spacecraft),
    target: GameObjectBody,
) -> Vec<GameCmd> {
    let mut result = vec![];
    for (component_id, component) in &spacecraft.components {
        if let Component::Weapon(weapon) = component {
            if !weapon.active {
                result.push(GameCmd::ExecuteComponentCmd(
                    *spacecraft_id,
                    *component_id,
                    ComponentCmd::SetActive(true),
                ));
            }
            let world_position = spacecraft
                .body
                .relative_to_world(weapon.body.centered_position() - spacecraft.center_of_mass);
            let direction = target.position - world_position;
            let angle = Vec2::from_angle(
                weapon.body.orientation.to_radians() + spacecraft.body.rotation,
            )
            .angle_between(direction);
            result.push(GameCmd::ExecuteComponentCmd(
                *spacecraft_id,
                *component_id,
                ComponentCmd::SetRotation(angle),
            ));
        }
    }
    result
}

pub fn predictive_shoot_at(
    (spacecraft_id, spacecraft): (&GameObjectId, &Spacecraft),
    target: GameObjectBody,
) -> Vec<GameCmd> {
    let mut result = vec![];
    for (component_id, component) in &spacecraft.components {
        if let Component::Weapon(weapon) = component {
            let weapon_world_pos = spacecraft
                .body
                .relative_to_world(weapon.body.centered_position() - spacecraft.center_of_mass);
            let relative_pos = target.position - weapon_world_pos; // relative position of target
            let relative_vel = target.velocity - spacecraft.body.velocity; // relative velocity of target
            let projectile_speed = weapon.projectile_speed;

            let a = relative_vel.x * relative_vel.x + relative_vel.y * relative_vel.y
                - projectile_speed * projectile_speed;
            let b = 2. * (relative_pos.x * relative_vel.x + relative_pos.y * relative_vel.y);
            let c = relative_pos.x * relative_pos.x + relative_pos.y * relative_pos.y;

            let d = b * b - 4. * a * c;

            let set_active = |activate, result: &mut Vec<GameCmd>| {
                if activate != weapon.active {
                    result.push(GameCmd::ExecuteComponentCmd(
                        *spacecraft_id,
                        *component_id,
                        ComponentCmd::SetActive(activate),
                    ));
                }
            };

            if d < 0. {
                set_active(false, &mut result);
                continue;
            }
            let t1 = (-b + d.sqrt()) / (2. * a);
            let t2 = (-b - d.sqrt()) / (2. * a);
            let (t1, t2) = (t1.min(t2), t1.max(t2));

            let t = if t1 > 0. { t1 } else { t2 };

            if t < 0. {
                set_active(false, &mut result);
                continue;
            }

            let result_direction = (relative_pos + relative_vel * t) / t;
            let target_weapon_rotation = result_direction.angle()
                - (spacecraft.body.rotation + weapon.body.orientation.to_radians());

            if (weapon.rotation - target_weapon_rotation).abs() > 0.05 {
                result.push(GameCmd::ExecuteComponentCmd(
                    *spacecraft_id,
                    *component_id,
                    ComponentCmd::SetRotation(target_weapon_rotation),
                ));
            }

            set_active(true, &mut result);
        }
    }
    result
}

/// Rotation speed is a multiplies rot_offset
pub fn impulse_fly_to(
    (spacecraft_id, spacecraft): (&GameObjectId, &Spacecraft),
    target: GameObjectBody,
    rotation_speed: f32,
) -> Vec<GameCmd> {
    let mut result = vec![];

    let final_direction = target.position - spacecraft.body.position;
    let final_velocity_direction = (final_direction - spacecraft.body.velocity).normalize();

    let spacecraft_optimal_direction = optimal_direction(&spacecraft);

    let rotation_offset = final_velocity_direction.angle_between(
        Vec2::from_angle(spacecraft.body.rotation).rotate(spacecraft_optimal_direction),
    );
    if rotation_offset.abs() > 0.1 {
        return rotate_to_direction(
            (spacecraft_id, spacecraft),
            final_velocity_direction.rotate(-spacecraft_optimal_direction),
            rotation_speed,
        );
    }
    for (component_id, component) in &spacecraft.components {
        if let Component::Engine(engine) = &component {
            let component_direction = Vec2::from_angle(
                engine.body.orientation.to_radians() + spacecraft.body.rotation,
            );

            let power = component_direction.dot(final_velocity_direction);

            let activate = power > 0.0;
            if engine.active != activate {
                result.push(GameCmd::ExecuteComponentCmd(
                    *spacecraft_id,
                    *component_id,
                    ComponentCmd::SetActive(activate),
                ));
            }
            if activate {
                result.push(GameCmd::ExecuteComponentCmd(
                    *spacecraft_id,
                    *component_id,
                    ComponentCmd::SetPower(power),
                ));
            }
        }
    }

    result
}

fn optimal_direction(spacecraft: &Spacecraft) -> Vec2 {
    let mut result = Vec2::ONE;
    let bench = |dir: Vec2| -> f32 {
        let mut result = 0.;
        for component in spacecraft.components.values() {
            if let Component::Engine(engine) = component {
                let component_direction = Vec2::from_angle(
                    engine.body.orientation.to_radians(),
                );
                let alignment = component_direction.dot(dir);
                if alignment > 0. {
                    let strength = alignment*engine.thrust;
                    result += strength;
                }
            }
        }
        result
    };
    for i in (0..360).step_by(10) {
        let cur_angle = i as f32/360.*2.*PI;
        let cur_dir = Vec2::from_angle(cur_angle);
        if bench(result) < bench(cur_dir) {
            result = cur_dir;
        }
    }
    let result_angle = result.angle();
    for i in -10..10 {
        let angle_offs = i as f32 / 360. * 2.*PI;
        let cur_dir = Vec2::from_angle(result_angle+angle_offs);
        if bench(result) < bench(cur_dir) {
            result = cur_dir;
        }
    }
    result
}

pub fn fly_to(
    (spacecraft_id, spacecraft): (&GameObjectId, &Spacecraft),
    target: GameObjectBody,
) -> Vec<GameCmd> {
    let mut result = vec![];

    let target_direction = target.position - spacecraft.body.position;
    let target_velocity_direction =
        (target_direction - spacecraft.body.velocity).normalize();

    for (component_id, component) in &spacecraft.components {
        if let Component::Engine(engine) = &component {
            let component_orientation = Vec2::from_angle(
                engine.body.orientation.to_radians() + spacecraft.body.rotation,
            );

            let power = component_orientation.dot(target_velocity_direction);

            let activate = power > 0.0;
            if engine.active != activate {
                result.push(GameCmd::ExecuteComponentCmd(
                    *spacecraft_id,
                    *component_id,
                    ComponentCmd::SetActive(activate),
                ));
            }
            if activate && (engine.power - power).abs() > 0.1 {
                result.push(GameCmd::ExecuteComponentCmd(
                    *spacecraft_id,
                    *component_id,
                    ComponentCmd::SetPower(power),
                ));
            }
        }
    }

    result
}

pub fn rotate_to_game_object_body(
    spacecraft: (&GameObjectId, &Spacecraft),
    target: GameObjectBody,
    speed: f32,
) -> Vec<GameCmd> {
    let direction = (target.position - spacecraft.1.body.position).normalize();

    rotate_to_direction(spacecraft, direction, speed)
}

pub fn rotate_to_direction(
    spacecraft: (&GameObjectId, &Spacecraft),
    direction: Vec2,
    speed: f32,
) -> Vec<GameCmd> {
    let mut result = vec![];

    let (spacecraft_id, spacecraft) = spacecraft;

    let rot_offset = Vec2::from_angle(spacecraft.body.rotation).angle_between(direction);
    let target_angular_velocity = rot_offset * speed;
    let angular_velocity_offset = target_angular_velocity - spacecraft.body.angular_velocity;

    for (component_id, component) in &spacecraft.components {
        if let Component::Engine(engine) = &component {
            let offset = engine.body.centered_position() - spacecraft.center_of_mass;
            let direction = Vec2::from_angle(engine.body.orientation.to_radians());

            let rot_effect = offset.perp_dot(direction);

            let activate = rot_effect.signum() == angular_velocity_offset.signum();
            if engine.active != activate {
                result.push(GameCmd::ExecuteComponentCmd(
                    *spacecraft_id,
                    *component_id,
                    ComponentCmd::SetActive(activate),
                ));
            }

            if activate && engine.power != 1. {
                result.push(GameCmd::ExecuteComponentCmd(
                    *spacecraft_id,
                    *component_id,
                    ComponentCmd::SetPower(1.),
                ));
            }
        }
    }

    result
}

pub fn deactivate_weapons(spacecraft: (&GameObjectId, &Spacecraft)) -> Vec<GameCmd> {
    let mut result = vec![];

    let (spacecraft_id, spacecraft) = spacecraft;

    for (component_id, component) in &spacecraft.components {
        if let Component::Weapon(weapon) = &component {
            if weapon.active {
                result.push(GameCmd::ExecuteComponentCmd(
                    *spacecraft_id,
                    *component_id,
                    ComponentCmd::SetActive(false),
                ));
            }
        }
    }

    result
}

pub fn achieve_velocity(
    (spacecraft_id, spacecraft): (&GameObjectId, &Spacecraft),
    target_velocity: Vec2,
) -> Vec<GameCmd> {
    let mut result = vec![];

    let velocity_offset = target_velocity - spacecraft.body.velocity;


    
    result
}