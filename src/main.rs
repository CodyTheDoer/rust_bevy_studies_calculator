/*
To Do:
Attach input function to camera motion
   scroll wheel: zoom in or out 
Ray tracing and interface with mesh, need to pick algorithim, looking at Watertight Ray/Triangle Intersection
figure out button click animation
build screen or figure out how to attach text to existing screen component in demo.
*/

use std::f32::consts::*;

use bevy::input::common_conditions::*;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::*;
use glam::Vec3, 

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


#[derive(Debug, Component)]
struct Calculator;

#[derive(Debug, Component)]
struct Player;

#[derive(Debug, Component)]
struct WorldModelCamera;

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

fn watertight_ray_triangle_intersection(// Fed Ray origin, Direction, and triangle, returns true if intersects
    origin: Vec3,                   // Ray origin
    direction: Vec3,                // Ray Direction
    triangle: (Vec3, Vec3, Vec3),   // Triangle contains coordinates for each vertex in order to wind triangle for testing
    backface_culling: bool,         // Tracks whether or not bf culling is enabled on this triangle
) -> bool {

    // calculate dimension where the ray direction is maximal 
    fn index_max_abs_dim(v: Vec3) -> usize {
        let abs_v = v.abs()
        if abs_v.x() >= abs_v.y() && abs_v.x() >= abs_v.z() {
            0 // returns x index
        } else if abs_v.y() >= abs_v.x() && abs_v.y() >= abs_v.z() {
            1 // returns y index
        } else {
            2 // returns z index
        }
    }

        // int kz = max_dim(abs(dir));
    let kz = index_max_abs_dim(direction);
    
        // int kx = kz+1; if (kx == 3) kx = 0;
        // int ky = kx+1; if (ky == 3) ky = 0;
    let mut kx = (kz + 1) % 3;
    let mut ky = (kx + 1) % 3;

        // if (dir[kz] < 0.0f) swap(kx,ky);
    // swap kx and ky dimension to preserve winding direction of triangles 
    if direction[kz] < 0.0 {
        std::mem::swap(&mut kx, &mut ky);
    }

        // float Sx = dir[kx]/dir[kz];
        // float Sy = dir[ky]/dir[kz];
        // float Sz = 1.0f/dir[kz];
    // calculate shear constants
    let valid_kz = direction[kz].abs() > std::f64::EPSILON; //Ensure we're not dividing against zero
    if !valid_kz { // Early return if direction[kz] is too close to zero
        return false;
    }

    let sx: f32 = if valid_kz {direction[kx] / direction[kz]} else {panic!("Placeholder Responce: sx: ")};
    let sy: f32 = if valid_kz {direction[ky] / direction[kz]} else { panic!("Placeholder Responce: sy: ")};
    let sz: f32 = if valid_kz {1.0 / direction[kz]} else {panic!("Placeholder Responce: sz: ")};
    
        // const Vec3f A = tri.A-org;
        // const Vec3f B = tri.B-org;
        // const Vec3f C = tri.C-org;
    // Calculate vertices relative to ray origin
    const POINT_A = triangle.0 - origin;
    const POINT_B = triangle.1 - origin;
    const POINT_C = triangle.2 - origin;

        // const float Ax = A[kx] - Sx*A[kz];
        // const float Ay = A[ky] - Sy*A[kz];
        // const float Bx = B[kx] - Sx*B[kz];
        // const float By = B[ky] - Sy*B[kz];
        // const float Cx = C[kx] - Sx*C[kz];
        // const float Cy = C[ky] - Sy*C[kz];
    // perform shear and scale of vertices
    const POINT_A_X: f32 = POINT_A[kx] - sx * POINT_A[kz];
    const POINT_A_Y: f32 = POINT_A[ky] - sy * POINT_A[kz];

    const POINT_B_X: f32 = POINT_B[kx] - sx * POINT_B[kz];
    const POINT_B_Y: f32 = POINT_B[ky] - sy * POINT_B[kz];
    
    const POINT_C_X: f32 = POINT_C[kx] - sx * POINT_C[kz];
    const POINT_C_Y: f32 = POINT_C[ky] - sy * POINT_C[kz];
    

        // float U = Cx*By - Cy*Bx;
        // float V = Ax*Cy - Ay*Cx;
        // float W = Bx*Ay - By*Ax;
    // Calculate scaled barycentric coordinates
    let mut u: f32 = POINT_C_X * POINT_B_Y - POINT_C_Y * POINT_B_X;
    let mut v: f32 = POINT_A_X * POINT_C_Y - POINT_A_Y * POINT_C_X;
    let mut w: f32 = POINT_B_X * POINT_A_Y - POINT_B_Y * POINT_A_X;

        // if (U == 0.0f || V == 0.0f || W == 0.0f) {
        //     double CxBy = (double)Cx*(double)By;
        //     double CyBx = (double)Cy*(double)Bx;
        //     U = (float)(CxBy - CyBx);
        //     double AxCy = (double)Ax*(double)Cy;
        //     double AyCx = (double)Ay*(double)Cx;
        //     V = (float)(AxCy - AyCx);
        //     double BxAy = (double)Bx*(double)Ay;
        //     double ByAx = (double)By*(double)Ax;
        //     W = (float)(BxAy - ByAx);
        // }    
    // Fallback to test against edges using double precision
    if (u == 0.0 || v == 0.0 || w == 0.0) {
        let cx_by: f64 = f64::from(point_c_x) * f64::from(point_b_y);
        let cy_bx: f64 = f64::from(point_c_y) * f64::from(point_b_x);
        u = (cx_by - cy_bx) as f32;
        let ax_cy: f64 = f64::from(point_a_x) * f64::from(point_c_y);
        let ay_cx: f64 = f64::from(point_a_y) * f64::from(point_c_x);
        v = (ax_cy - ay_cx) as f32;
        let bx_ay: f64 = f64::from(point_b_x) * f64::from(point_a_y);
        let by_ax: f64 = f64::from(point_b_y) * f64::from(point_a_x);
        w = (bx_ay - by_ax) as f32;
    }

        // #ifdef BACKFACE_CULLING
        // if (U<0.0f || V<0.0f || W<0.0f) return;
        // #else
        // if ((U<0.0f || V<0.0f || W<0.0f) &&
        // (U>0.0f || V>0.0f || W>0.0f)) return;
        // #endif
    // Perform edge tests. Moving this test before and at the end of the previous conditional gives higher performance.
    fn perform_edge_tests(u: f32, v: f32, w: f32, backface_culling: bool) -> bool {
        if backface_culling == true {
            if (u < 0.0 || v < 0.0 || w < 0.0) {
                return false;
            } 
        } else {
            if ((u < 0.0 || v < 0.0 || w < 0.0) && (u > 0.0 || v > 0.0 || w > 0.0)) {
                return false;
            }
        }
        return true;
    }

    if !perform_edge_tests(u, v, w, backface_culling) {
        return false;
    }

        // float det = U+V+W;
        // if (det == 0.0f) return;

    // Calculate the determinate
    let determinate: f32 = u + v + w;
    if determinate == 0.0 {
        return false
    }

        // const float Az = Sz*A[kz];
        // const float Bz = Sz*B[kz];
        // const float Cz = Sz*C[kz];
        // const float T = U*Az + V*Bz + W*Cz;
    // Calculate scaled z-coordinates of vertices and use them to calculate the hit distance.
    const POINT_A_Z: f32 = sz * POINT_A[kz];
    const POINT_B_Z: f32 = sz * POINT_B[kz];
    const POINT_C_Z: f32 = sz * POINT_C[kz];
    const T: f32 = u * POINT_A_Z + v * POINT_B_Z + w * POINT_C_Z;
    

        // const float rcpDet = 1.0f/det;
        // hit.u = U*rcpDet;
        // hit.v = V*rcpDet;
        // hit.w = W*rcpDet;
        // hit.t = T*rcpDet;
    // Normalize U, V, W, and T
    const RECIPROCAL_OF_DETERMINATE: f32 = 1.0 / determinate;
    let hit_u: f32 = u * RECIPROCAL_OF_DETERMINATE;
    let hit_v: f32 = v * RECIPROCAL_OF_DETERMINATE;
    let hit_w: f32 = w * RECIPROCAL_OF_DETERMINATE;
    let hit_t: f32 = T * RECIPROCAL_OF_DETERMINATE;

        // #ifdef BACKFACE_CULLING
        // if (T < 0.0f || T > hit.t * det)
        // return;
        // #else
        // int det_sign = sign_mask(det);
        // if (xorf(T,det_sign) < 0.0f) ||
        // xorf(T,det_sign) > hit.t * xorf(det, det_sign))
        // return;
        // #endif
    
    fn sign_mask(f: f32) -> u32 {
        (f.to_bits() >> 31) & 1 // returns 1 if f is negative, and 0 if positive
    }
    fn xorf(value: f32, sign_mask: u32) -> f32 {
        let value_bits = value.to_bits();
        let result_bits = value_bits ^ (sign_mask << 31);
        f32::from_bits(result_bits) // returns value with flipped sign if determinate is negative.
    }

    if backface_culling == true {
        if (t < 0.0 || t > hit_t * determinate) {
            return false;
        } else {
            let determinate_sign = sign_mask(determinate);
            let t_xor = xorf(t, determinate_sign)
            let determinate_xor = xorf(determinate, determinate_sign)

            if t_xor < 0.0 || t_xor > hit_t * determinate_xor {
                return false
            }
        }
    }

        // Vec3i nearID(0,1,2), farID(3,4,5);
        // int nearX = nearID[kx], farX = farID[kx];
        // int nearY = nearID[ky], farY = farID[ky];
        // int nearZ = nearID[kz], farZ = farID[kz];
        // if (dir[kx] < 0.0f) swap(nearX,farX);
        // if (dir[ky] < 0.0f) swap(nearY,farY);
        // if (dir[kz] < 0.0f) swap(nearZ,farZ);
    // Calculate the offset to the newar and far planes for the kx, ky, and kz dimensions for a 
    // box stored in the order lower_x, lower_y, lower_z, upper_x, upper_y, upper_z in memory.
    let near_id: [i32, 3] = [0, 1, 2];
    let far_id: [i32, 3] = [3, 4, 5];

    let mut near_x: f32 = near_id[kx];
    let mut far_x: f32 = far_id[kx];

    let mut near_y: f32 = near_id[ky];
    let mut far_y: f32 = far_id[ky];

    let mut near_z: f32 = near_id[kz];
    let mut far_z: f32 = far_id[kz];

    if direction[kx] < 0.0 {std::mem::swap(&mut near_x, &mut far_x);}
    if direction[ky] < 0.0 {std::mem::swap(&mut near_y, &mut far_y);}
    if direction[kz] < 0.0 {std::mem::swap(&mut near_z, &mut far_z);}

        // float p = 1.0f + 2^-23;
        // float m = 1.0f - 2^-23;

        // float up(float a) { return a>0.0f ? a*p : a*m; }
        // float dn(float a) { return a>0.0f ? a*m : a*p; }

        // float Up(float a) { return a*p; }
        // float Dn(float a) { return a*m; }
    // up and down rounding.
    let p: f32 = 1.0 + (2.0f32).powi(-23);
    let m: f32 = 1.0 - (2.0f32).powi(-23);

    fn round_up_check_sign(num: f32, p: f32, m: f32) -> f32 {
        if num > 0.0 {
            num * p
        } else {
            num * m
        }
    }

    fn round_down_check_sign(num: f32, p: f32, m: f32) -> f32 {
        if num > 0.0 {
            num * m
        } else {
            num * p
        }
    }
    // Fast rounding for positive numbers
    fn round_up_positive(num: f32, p: f32) -> f32 {
        num * p
    }

    fn round_down_positive(num: f32, m: f32) -> f32 {
        num * m
    }
    
        // const float eps = 5.0f * 2^-24;
        // Vec3f lower = Dn(abs(org-box.lower));
        // Vec3f upper = Up(abs(org-box.upper));
    // Calculate corrected origin for new and far plane distance calculations. Each floating point
    // operation is forced to be rounded into the correct direction.
    const EPSILON: f32 = 5.0 * (2.0f32).powi(-24);

    let bounding_box_lower = Vec3::new(
        point_a.x().min(point_b.x()).min(point_c.x());
        point_a.y().min(point_b.y()).min(point_c.y());
        point_a.z().min(point_b.z()).min(point_c.z());
    )
    let bounding_box_upper = Vec3::new(
        point_a.x().max(point_b.x()).max(point_c.x()),
        point_a.y().max(point_b.y()).max(point_c.y()),
        point_a.z().max(point_b.z()).max(point_c.z()),
    );

    let lower_bounds round_down_positive((origin - bounding_box_lower).abs());
    let upper_bounds round_up_positive((origin - bounding_box_upper).abs());

        // float max_z = max(lower[kz],upper[kz]);
        // float err_near_x = Up(lower[kx]+max_z);
        // float err_near_y = Up(lower[ky]+max_z);
    let max_z: f32 = bounding_box_lower[kz].max(bounding_box_upper[kz]);
    let error_near_x: f32 = round_up_positive(bounding_box_lower[kx] + max_z, p);
    let error_near_y: f32 = round_up_positive(bounding_box_lower[ky] + max_z, p);

        // float org_near_x = up(org[kx]+Up(eps*err_near_x));
        // float org_near_y = up(org[ky]+Up(eps*err_near_y));
        // float org_near_z = org[kz];
    let origin_near_x = round_up_check_sign(origin[kx] + EPSILON * error_near_x, p, m);
    let origin_near_y = round_up_check_sign(origin[ky] + EPSILON * error_near_y, p, m);
    let origin_near_z = origin[kz];










    float err_far_x = Up(upper[kx]+max_z);
    float err_far_y = Up(upper[ky]+max_z);
    float org_far_x = dn(org[kx]-Up(eps*err_far_x));
    float org_far_y = dn(org[ky]-Up(eps*err_far_y));
    float org_far_z = org[kz];
    if (dir[kx] < 0.0f) swap(org_near_x,org_far_x);
    if (dir[ky] < 0.0f) swap(org_near_y,org_far_y);

    // Calculate corrected reciprocal direction for near and far plane distance calculations. We
    // correct with one additional ulp to also correctly round the subtraction inside the traversal
    // loop. The works only because the ray is only allowed to hit geometry in front of it.
    float rdir_near_x = Dn(Dn(rdir[kx]));
    float rdir_near_y = Dn(Dn(rdir[ky]));
    float rdir_near_z = Dn(Dn(rdir[kz]))
    float rdir_far_x = Up(Up(rdir[kx]));
    float rdir_far_y = Up(Up(rdir[ky]));
    float rdir_far_z = Up(Up(rdir[kz]));
    float tNearX = (box[nearX] - org_near_x) * rdir_near_x;
    float tNearY = (box[nearY] - org_near_y) * rdir_near_y;
    float tNearZ = (box[nearZ] - org_near_z) * rdir_near_z;
    float tFarX = (box[farX ] - org_far_x ) * rdir_far_x;
    float tFarY = (box[farY ] - org_far_y ) * rdir_far_y;
    float tFarZ = (box[farZ ] - org_far_z ) * rdir_far_z;
    float tNear = max(tNearX,tNearY,tNearZ,rayNear);
    float tFar = min(tFarX ,tFarY ,tFarZ ,rayFar );
    bool hit = tNear <= tFar;
}