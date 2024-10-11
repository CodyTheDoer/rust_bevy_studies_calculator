/*
To Do:
Attach input function to camera motion
   scroll wheel: zoom in or out 
Ray tracing and interface with mesh, need to pick algorithim, looking at Watertight or possible the Möller-Trumbore algorithm
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

// Structs

#[derive(Debug, Component)]
struct Calculator;

#[derive(Debug, Component)]
struct Player;

#[derive(Debug, Component)]
struct WorldModelCamera;

// Calculator Library Functions

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

fn watertight_ray_triangle_intersection(
    orig: Vec3,     // Ray Origin
    dir: Vec3,      // Ray Direction
    v0: Vec3,       // Triangle Vertex 0
    v1: Vec3,       // Triangle Vertex 2
    v2: Vec3,       // Triangle Vertex 3
) -> option<(f32, f32, f32)> {
    let kz = dir.abs().max_element_index();
    let kx = (kz + 1) % 3;
    let ky = (kx + 1) % 3;

    // Reorder the ray direction and origin based on max 
    let dir = dir.permute(kx, ky, kz);
    let orig = orig.permute(kx, ky, kz);

    //compute the triangle edges
    let mut v0t = v0 - orig;
    let mut v1t = v1 - orig;
    let mut v2t = v2 - orig;

    //permute triangle vertices
    v0t = v0t.permute(kx, ky, kz);
    v1t = v1t.permute(kx, ky, kz);
    v2t = v2t.permute(kx, ky, kz);

    // Edge Setup
    let e0 = v1t - v0t;
    let e1 = v2t - v0t;

    // Calc the determinant and scale it by the correct component of the ray direction
    let det = e0.x * e1.y - e0.y * e1.x;
    if det.abs() < 1e-8 {
        return None; // Ray is parallel to triangle
    }

    // Calc barycentric coordinates
    let inv_det = 1.0 / det;
    let t = (v0t.x * (v0t.y - dir.y) - v0t.y * (v0t.x - dir.x)) * inv_det;
    let u = (dir.x * e1.y - dir.y * e1.x) * inv_det;
    let v = (e0.x * dir.y - e0.y * dir.x) * inv_det;

    // If the barycentric coords are out of the triangle there is no intersection
    if u < 0.0 || v < 0.0 || (u + v) > 1.0 {
        return None;
    }

    // If we get this far we have a valid intersection
    Some((t, u, v))
}

fn draw_cursor(
    camera_query: Query<(&camera, &GlobalTransform)>,
    calculator_query: Query<&GlobalTransform, With<Calculator>>,
    windows: Query<&Window>
    mut gizmos: Gizmos
) {
    let (camera, camera_transform) = camera_query.single();
    let calculator = calculator_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    }

    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    }

    for triangle in calculator_triangles.iter() {
        let (v0, v1, v2) = triangle;
        if let Some((t, u, v)) = watertight_ray_triangle_intersection(ray.origin, ray.direction, *v0, *v1, *v2) {
            // Successfully intersected with mesh
            let point = ray.get_point(t);
            gizmos.circle(point, calculator.up(), 0.2, Color::WHITE);
            break;
        }
    }
}

// fn draw_cursor(
//     camera_query: Query<(&Camera, &GlobalTransform)>,
//     calculator_query: Query<&GlobalTransform, With<Calculator>>,
//     windows: Query<&Window>,
//     mut gizmos: Gizmos,
// ) {
//     let (camera, camera_transform) = camera_query.single();
//     let calculator = calculator_query.single();

//     let Some(cursor_position) = windows.single().cursor_position() else {
//         return;
//     };

//     // Calculate a ray pointing from the camera into the world based on the cursor's position.
//     let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
//         return;
//     };

//     // Calculate if and where the ray is hitting the calculator.
//     let Some(distance) =
//         ray.intersect_plane(calculator.translation(), InfinitePlane3d::new(calculator.up()))
//     else {
//         return;
//     };
//     let point = ray.get_point(distance);

//     // Draw a circle just above the calculator at that position.
//     gizmos.circle(point + calculator.up() * 0.01, calculator.up(), 0.2, Color::WHITE);
// }


fn setup_calculator_glb(
    mut commands: Commands, 
    asset_server: Res<AssetServer>
    mut meshes: ResMut<Assets<Mesh>>
) {
    let calculator_handle = asset_server.load("Calculator.glb#Scene0");
    commands.spawn((
        SceneBundle {
            scene: calculator_handle.clone(),
            ..default()
        },
        Calculator,
    ));
}

// fn setup_calculator_glb(mut commands: Commands, asset_server: Res<AssetServer>) {
//     commands.spawn((
//         SceneBundle {
//             scene: asset_server.load("Calculator.glb#Scene0"), // Load the scene from GLB file
//             ..default()
//         },
//         Calculator,  // Tag it with Ground for raycasting detection
//     ));
// }

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

