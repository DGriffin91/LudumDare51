use std::time::Duration;

use bevy::prelude::*;

use crate::{
    assets::ModelAssets,
    enemies::{Enemy, Health},
};

#[derive(Clone, Component)]
pub enum Turret {
    Laser(Entity),
    Shockwave(Entity),
}

#[derive(Component, Deref, DerefMut)]
pub struct AttackDamage(pub f32);

#[derive(Component, Deref, DerefMut)]
pub struct Cooldown(pub Timer);

#[derive(Component, Deref, DerefMut)]
pub struct Range(f32);

impl Turret {
    pub fn spawn_laser_turret(
        com: &mut Commands,
        trans: Vec3,
        model_assets: &ModelAssets,
    ) -> Turret {
        let mut entity = com.spawn_bundle(SceneBundle {
            scene: model_assets.shockwave_unit.clone(),
            transform: Transform::from_translation(trans),
            ..Default::default()
        });
        let t = Turret::Shockwave(entity.id());
        entity
            .insert(AttackDamage(0.1))
            .insert(Cooldown(Timer::new(Duration::from_secs_f32(0.5), true)))
            .insert(Range(2.0))
            .insert(t.clone());
        t
    }

    pub fn entity(&self) -> Entity {
        match self {
            Turret::Laser(u) => *u,
            Turret::Shockwave(u) => *u,
        }
    }

    pub fn despawn(&self, com: &mut Commands) {
        com.entity(self.entity()).despawn_recursive();
    }
}

pub fn apply_damage(
    time: Res<Time>,
    mut turrets: Query<(&Transform, &AttackDamage, &Range, &mut Cooldown), With<Turret>>,
    mut enemies: Query<(&Transform, &mut Health), With<Enemy>>,
) {
    for (turret_trans, damage, range, mut cooldown) in turrets.iter_mut() {
        cooldown.tick(time.delta());
        for (enemy_trans, mut health) in enemies.iter_mut() {
            if enemy_trans.translation.distance(turret_trans.translation) < **range {
                if cooldown.finished() {
                    cooldown.reset();
                    health.0 -= damage.0;
                }
            }
        }
    }
}
