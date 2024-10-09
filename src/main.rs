/*
To Do:
Attach input function to camera motion
   held right click: engage camera motion 
   scroll wheel: zoom in or out 
figure out button click animation
build screen or figure out how to attach text to existing screen component in demo.
*/

use std::f32::consts::*;

use bevy::input::common_conditions::*;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_systems(
            Startup,
            (
                setup_calculator_glb,
                spawn_view_model,
                spawn_lights,
                spawn_text,
                add,
                subtract,
                multiply,
                divide,
            ),
        )
        .add_systems(
            Update, 
            (
                adjust_player_camera
                .run_if(input_pressed(MouseButton::Right)),
                draw_cursor,
                change_fov,
                animate_light_direction,
            ),
        )
        .run();
}

// Calculator Functionality

fn add() {
    let result = bevy_calculator::add(24, 49);
    println!("add result: {:?}", result)
}

fn subtract() {
    let result = bevy_calculator::subtract(24, 49);
    println!("subtract result: {:?}", result)
}

fn divide() {
    let result = bevy_calculator::divide(24, 49);
    println!("divide result: {:?}", result)
}

fn multiply() {
    let result = bevy_calculator::multiply(24, 49);
    println!("multiply result: {:?}", result)
}

// GUI Functionality

fn adjust_player_camera(
    mut mouse_motion: EventReader<MouseMotion>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    let mut transform = player.single_mut();
    for motion in mouse_motion.read() {
        let yaw = -motion.delta.x * 0.003;
        let pitch = motion.delta.y * 0.002;
        // Order of rotations is important, see <https://gamedev.stackexchange.com/a/136175/103059>
        transform.rotate_y(yaw);
        transform.rotate_local_x(pitch);
    }
}

fn change_fov(input: Res<ButtonInput<KeyCode>>,mut world_model_projection: Query<&mut Projection, With<WorldModelCamera>>) {
    let mut projection = world_model_projection.single_mut();
    let Projection::Perspective(ref mut perspective) = projection.as_mut() else {
        unreachable!(
            "The `Projection` component was explicitly built with `Projection::Perspective`"
        );
    };

    if input.pressed(KeyCode::ArrowUp) {
        perspective.fov -= 1.0_f32.to_radians();
        perspective.fov = perspective.fov.max(20.0_f32.to_radians());
    }
    if input.pressed(KeyCode::ArrowDown) {
        perspective.fov += 1.0_f32.to_radians();
        perspective.fov = perspective.fov.min(160.0_f32.to_radians());
    }
}

fn draw_cursor(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    calculator_query: Query<&GlobalTransform, With<Calculator>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();
    let calculator = calculator_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Calculate if and where the ray is hitting the calculator.
    let Some(distance) =
        ray.intersect_plane(calculator.translation(), InfinitePlane3d::new(calculator.up()))
    else {
        return;
    };
    let point = ray.get_point(distance);

    // Draw a circle just above the calculator at that position.
    gizmos.circle(point + calculator.up() * 0.01, calculator.up(), 0.2, Color::WHITE);
}


// GUI Backend
#[derive(Debug, Component)]
struct Calculator;

#[derive(Debug, Component)]
struct Player;

#[derive(Debug, Component)]
struct WorldModelCamera;


fn setup_calculator_glb(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SceneBundle {
            scene: asset_server.load("Calculator.glb#Scene0"), // Load the scene from GLB file
            ..default()
        },
        Calculator,  // Tag it with Ground for raycasting detection
    ));
}

fn spawn_view_model(
    mut commands: Commands,
) {
    commands
        .spawn((
            Player,
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 7.0, 5.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                WorldModelCamera,
                Camera3dBundle {
                    projection: PerspectiveProjection {
                        fov: 90.0_f32.to_radians(),
                        ..default()
                    }
                    .into(),
                    ..default()
                },
            ));
        });
}

fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                ..default()
            },
            cascade_shadow_config: CascadeShadowConfigBuilder {
                num_cascades: 1,
                maximum_distance: 1.6,
                ..default()
            }
        .into(),
        ..default()
        },
    ));
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}

fn spawn_text(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                concat!(
                    "Move the camera with your mouse holding right click to enable movement.\n",
                    "Press arrow up to decrease the FOV of the world model.\n",
                    "Press arrow down to increase the FOV of the world model."
                ),
                TextStyle {
                    font_size: 25.0,
                    ..default()
                },
            ));
        });
}