
use std::f32::consts::*;

use bevy::input::common_conditions::*;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::*;

use glam::Vec3;

struct Hit {
    u: f32,
    v: f32,
    w: f32,
    t: f32,
}

#[derive(Debug, Component)]
struct GlbComponent;

#[derive(Debug, Component)]
struct Player;

#[derive(Debug, Component)]
struct WorldModelCamera;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .insert_resource(DirectionalLightShadowMap { size: 4096 })
    .add_systems(
        Startup,
        (   
            example_triangle_ray_test,
            setup_glb,
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

fn watertight_ray_triangle_intersection(
    origin: Vec3,                   // Ray origin
    direction: Vec3,                // Ray direction
    triangle: (Vec3, Vec3, Vec3),   // Triangle vertices
    backface_culling: bool,         // Backface culling flag
) -> Option<Hit> {
    // Calculate dimension where the ray direction is maximal
    fn index_max_abs_dim(v: Vec3) -> usize {
        let abs_v = v.abs();
        if abs_v.x >= abs_v.y && abs_v.x >= abs_v.z {
            0
        } else if abs_v.y >= abs_v.x && abs_v.y >= abs_v.z {
            1
        } else {
            2
        }
    }

    let kz = index_max_abs_dim(direction);
    let mut kx = (kz + 1) % 3;
    let mut ky = (kx + 1) % 3;

    // Swap kx and ky to preserve winding direction of triangles
    if direction[kz] < 0.0 {
        std::mem::swap(&mut kx, &mut ky);
    }

    let f32_epsilon_check = std::f32::EPSILON;

    // Ensure we're not dividing by zero
    if direction[kz].abs() < f32_epsilon_check {
        return None;
    }

    // Calculate shear constants
    let sx: f32 = direction[kx] / direction[kz];
    let sy: f32 = direction[ky] / direction[kz];
    let sz: f32 = 1.0 / direction[kz];

    // Calculate vertices relative to ray origin
    let point_a = triangle.0 - origin;
    let point_b = triangle.1 - origin;
    let point_c = triangle.2 - origin;

    // Perform shear and scale of vertices
    let point_a_x = point_a[kx] - sx * point_a[kz];
    let point_a_y = point_a[ky] - sy * point_a[kz];
    let point_b_x = point_b[kx] - sx * point_b[kz];
    let point_b_y = point_b[ky] - sy * point_b[kz];
    let point_c_x = point_c[kx] - sx * point_c[kz];
    let point_c_y = point_c[ky] - sy * point_c[kz];

    // Calculate scaled barycentric coordinates
    let mut u = point_c_x * point_b_y - point_c_y * point_b_x;
    let mut v = point_a_x * point_c_y - point_a_y * point_c_x;
    let mut w = point_b_x * point_a_y - point_b_y * point_a_x;

    // Fallback to test against edges using double precision
    if u.abs() < f32_epsilon_check || v.abs() < f32_epsilon_check || w.abs() < f32_epsilon_check {
        let cx_by = (point_c_x as f64) * (point_b_y as f64);
        let cy_bx = (point_c_y as f64) * (point_b_x as f64);
        u = (cx_by - cy_bx) as f32;

        let ax_cy = (point_a_x as f64) * (point_c_y as f64);
        let ay_cx = (point_a_y as f64) * (point_c_x as f64);
        v = (ax_cy - ay_cx) as f32;

        let bx_ay = (point_b_x as f64) * (point_a_y as f64);
        let by_ax = (point_b_y as f64) * (point_a_x as f64);
        w = (bx_ay - by_ax) as f32;
    }

    // Calculate normal of the triangle to determine orientation
    let edge1 = triangle.1 - triangle.0;
    let edge2 = triangle.2 - triangle.0;
    let normal = edge1.cross(edge2);
    let facing = normal.dot(direction);

    // Log triangle orientation
    println!("Triangle normal: {:?}, Ray direction dot normal: {}", normal, facing);

    // Perform edge tests
    if backface_culling {
        if u < 0.0 || v < 0.0 || w < 0.0 {
            println!("Backface culling enabled: Ray hit the back of the triangle");
            return None;
        }
    } else {
        if (u < 0.0 || v < 0.0 || w < 0.0) && (u > 0.0 || v > 0.0 || w > 0.0) {
            return None;
        }
    }

    // Calculate determinant
    let mut det = u + v + w;
    if det == 0.0 {
        return None;
    }

    // Handle negative determinant
    if det < 0.0 {
        u = -u;
        v = -v;
        w = -w;
        det = -det;
    }

    // Calculate scaled z-coordinates of vertices and use them to calculate the hit distance
    let point_a_z = sz * point_a[kz];
    let point_b_z = sz * point_b[kz];
    let point_c_z = sz * point_c[kz];
    let mut t = u * point_a_z + v * point_b_z + w * point_c_z;

    // Apply sign flipping if necessary
    fn sign_mask(f: f32) -> u32 {
        (f.to_bits() >> 31) & 1 // returns 1 if f is negative, and 0 if positive
    }
    fn xorf(value: f32, sign_mask: u32) -> f32 {
        let value_bits = value.to_bits();
        let result_bits = value_bits ^ (sign_mask << 31);
        f32::from_bits(result_bits) // returns value with flipped sign if determinant is negative
    }

    let det_sign = sign_mask(det);
    t = xorf(t, det_sign);
    if t < 0.0 {
        return None;
    }

    // Normalize U, V, W, and T
    let reciprocal_det = 1.0 / det;
    let hit = Hit {
        u: u * reciprocal_det,
        v: v * reciprocal_det,
        w: w * reciprocal_det,
        t: t * reciprocal_det,
    };

    Some(hit)
}

fn example_triangle_ray_test() {
    // Example usage
    let origin = Vec3::new(0.0, 0.0, 0.0);
    let direction = Vec3::new(0.0, 0.0, 1.0);
    let triangle = (
        Vec3::new(1.0, 0.0, 5.0),
        Vec3::new(-1.0, 1.0, 5.0),
        Vec3::new(-1.0, -1.0, 5.0),
    );
    let backface_culling = true;

    if let Some(hit) = watertight_ray_triangle_intersection(origin, direction, triangle, backface_culling) {
        println!(
            "Intersection at t = {}, u = {}, v = {}, w = {}",
            hit.t, hit.u, hit.v, hit.w
        );
    } else {
        println!("No intersection");
    }

    // Test with reversed triangle winding
    let reversed_triangle = (
        Vec3::new(-1.0, -1.0, 5.0),
        Vec3::new(-1.0, 1.0, 5.0),
        Vec3::new(1.0, 0.0, 5.0),
    );

    if let Some(hit) = watertight_ray_triangle_intersection(origin, direction, reversed_triangle, backface_culling) {
        println!(
            "Intersection with reversed triangle at t = {}, u = {}, v = {}, w = {}",
            hit.t, hit.u, hit.v, hit.w
        );
    } else {
        println!("No intersection with reversed triangle");
    }
}

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
    glb_component_query: Query<&GlobalTransform, With<GlbComponent>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();
    let glb_component = glb_component_query.single();

    let Some(cursor_position) = windows.single().cursor_position() else {
        return;
    };

    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Calculate if and where the ray is hitting the glb_component.
    let Some(distance) =
        ray.intersect_plane(glb_component.translation(), InfinitePlane3d::new(glb_component.up()))
    else {
        return;
    };
    let point = ray.get_point(distance);

    // Draw a circle just above the glb_component at that position.
    gizmos.circle(point + glb_component.up() * 0.01, glb_component.up(), 0.2, Color::WHITE);
}

fn setup_glb(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SceneBundle {
            scene: asset_server.load("cube.glb#Scene0"), // Load the scene from GLB file
            ..default()
        },
        GlbComponent,  // Tag it for raycasting detection
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

// Calculator Calls
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
