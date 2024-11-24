use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::VectorSpace;
use bevy::prelude::*;
use rand::{thread_rng, Rng};

const ENEMY_SPEED: f32 = 5.0;
const ENEMY_SHOOTING_DISTANCE: f32 = 15.0;
const SPAWN_DURATION: f32 = 2.0;
const PROJECTILE_SPEED: f32 = 50.0;

#[derive(Component)]
struct Player {
    speed: f32,
}

#[derive(Component)]
struct Velocity(Vec3);
#[derive(Component)]
struct EnemyShip;

#[derive(Component)]
struct EnemyProjectile;

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct Rock;

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

#[derive(Resource)]
struct EnemyProjectileSpawnTimer(Timer);

fn main() {
    App::new()
        .insert_resource(EnemySpawnTimer(Timer::from_seconds(
            SPAWN_DURATION,
            TimerMode::Repeating,
        )))
        .add_plugins(DefaultPlugins)
        .add_plugins(PluginInit)
        .run();
}

struct PluginInit;

impl Plugin for PluginInit {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawnTimer(Timer::from_seconds(
            5.0,
            TimerMode::Repeating,
        )))
        .insert_resource(EnemyProjectileSpawnTimer(Timer::from_seconds(
            2.0,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, setup)
        .add_systems(Update, player_movement)
        .add_systems(Update, adjust_player_speed)
        .add_systems(Update, adjust_camera_distance)
        .add_systems(Update, player_rotation)
        .add_systems(Update, enemy_spawner)
        .add_systems(Update, enemy_shooting)
        .add_systems(Update, enemy_projectile_movement)
        .add_systems(Update, enemy_movement);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let camera = commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 50.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Y),
            ..default()
        })
        .insert(MainCamera)
        .id();

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: false,
            illuminance: 50000.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 1000.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Y),
        ..default()
    });

    commands.spawn(
        (SceneBundle {
            scene: asset_server.load("H:/Dev/Shi/assets/spacewithmeteors.glb#Scene0"),
            ..default()
        }),
    );

    for _i in 0..200 {
        commands.spawn((
            SceneBundle {
                scene: asset_server.load("H:/Dev/Shi/assets/meteorite.glb#Scene0"),
                transform: Transform::from_scale(Vec3::splat(thread_rng().gen_range(20.0..100.0)))
                    .with_translation(Vec3 {
                        x: thread_rng().gen_range(-2500.0..2500.0),
                        y: thread_rng().gen_range(-1000.0..0.0),
                        z: thread_rng().gen_range(-2500.0..2500.0),
                    }),
                ..default()
            },
            Rock,
        ));
    }

    let player_scene_handle: Handle<Scene> = asset_server.load("PlayerSpaceShip.glb#Scene0");
    commands
        .spawn(SceneBundle {
            scene: player_scene_handle,
            transform: Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(0.25)),
            ..default()
        })
        .insert(Player { speed: 5.0 })
        .add_child(camera);
}

fn player_movement(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Player), With<Player>>,
) {
    let time_step = time.delta_seconds();

    for (mut transform, player) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) {
            // Assuming 'W' key is used for moving forward
            let forward = transform.rotation.mul_vec3(Vec3::X * 1.0);
            let translation = forward * player.speed * time_step;
            transform.translation += translation;
        }

        transform.translation += direction * time_step * player.speed;
    }
}

fn adjust_player_speed(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&mut Player>,
) {
    for event in mouse_wheel_events.read() {
        for mut player in query.iter_mut() {
            if event.y > 0.0 {
                player.speed -= 2.5;
            } else if event.y < 0.0 {
                player.speed += 2.5;
            }

            // Clamp the speed to a minimum and maximum value, if desired
            player.speed = player.speed.clamp(0.0, 100.0);
        }
    }
}

fn adjust_camera_distance(
    query_player: Query<&Player>,
    mut query_camera: Query<&mut Transform, With<MainCamera>>,
) {
    if let Ok(player) = query_player.get_single() {
        if let Ok(mut transform) = query_camera.get_single_mut() {
            let desired_distance = 100.0 + ((player.speed / 5.0) * 20.0);
            transform.translation.y = desired_distance;
        }
    }
}

fn player_rotation(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut delta: Vec2 = Vec2::ZERO;
    for motion in mouse_motion_events.read() {
        delta += motion.delta;
    }

    // Sensitivity can be adjusted to your liking
    let sensitivity = Vec2::new(0.0025, 0.0025);

    for mut transform in query.iter_mut() {
        transform.rotate(Quat::from_rotation_y(-delta.x * sensitivity.x));
    }
}

fn enemy_spawner(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<EnemySpawnTimer>,
    asset_server: Res<AssetServer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();
        let enemy_ship_scene_handle: Handle<Scene> = asset_server.load("enemyship.glb#Scene0");
        commands
            .spawn(SceneBundle {
                scene: enemy_ship_scene_handle,
                transform: Transform::from_xyz(
                    rng.gen_range(-100.0..100.0),
                    0.0,
                    rng.gen_range(-100.0..100.0),
                ),
                ..default()
            })
            .insert(EnemyShip);
    }
}

fn enemy_movement(
    mut enemy_query: Query<&mut Transform, (With<EnemyShip>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<EnemyShip>)>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for mut enemy_transform in enemy_query.iter_mut() {
            let direction =
                (player_transform.translation - enemy_transform.translation).normalize();
            let distance = player_transform
                .translation
                .distance(enemy_transform.translation);

            if distance > ENEMY_SHOOTING_DISTANCE {
                enemy_transform.translation += direction * ENEMY_SPEED * time.delta_seconds();
            }

            let angle = Quat::from_rotation_arc(Vec3::X, direction);
            let smoothness = 2.0;
            enemy_transform.rotation = enemy_transform
                .rotation
                .slerp(angle, smoothness * time.delta_seconds());
        }
    }
}

fn enemy_shooting(
    mut commands: Commands,
    enemy_query: Query<&Transform, (With<EnemyShip>, Without<Player>)>,
    player_query: Query<&Transform, (With<Player>, Without<EnemyShip>)>,
    time: Res<Time>,
    mut timer: ResMut<EnemyProjectileSpawnTimer>,
    asset_server: Res<AssetServer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        if let Ok(player_transform) = player_query.get_single() {
            for enemy_transform in enemy_query.iter() {
                let distance = player_transform
                    .translation
                    .distance(enemy_transform.translation);

                if distance <= ENEMY_SHOOTING_DISTANCE {
                    //let bullet = asset_server.load("Bar4.png");
                    let direction =
                        (player_transform.translation - enemy_transform.translation).normalize();
                    let rotation = Quat::from_rotation_arc(Vec3::NEG_Z, direction);
                    let missile_handle: Handle<Scene> = asset_server.load("missile.glb#Scene0");

                    commands.spawn((
                        SceneBundle {
                            scene: missile_handle,
                            transform: Transform {
                                translation: enemy_transform.translation,
                                rotation: rotation,
                                ..default()
                            },
                            ..default()
                        },
                        EnemyProjectile,
                        Velocity(direction * PROJECTILE_SPEED),
                    ));
                }
            }
        }
    }
}

fn enemy_projectile_movement(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &Velocity), With<EnemyProjectile>>,
) {
    for (entity, mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0 * time.delta_seconds();

        if transform.translation.length() > 200.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}