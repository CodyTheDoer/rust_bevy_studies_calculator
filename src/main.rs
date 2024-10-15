/*
TODO:
Setup wrti logic in scene_handler_library and integrate the ray tracer
setup button click animations in blender
figure out how to load multiple gbl files into the scene_library_handler
figure out how to spawn text on the screen component of the calculator
*/

use scene_handler_library::{
    setup_glb, spawn_view_model, spawn_lights, animate_light_direction,
    draw_cursor, spawn_text, change_fov, adjust_player_camera,
}; 

use wrti_library::watertight_ray_triangle_intersection;

use glam::Vec3;

use bevy::input::common_conditions::*;
use bevy::pbr::DirectionalLightShadowMap;
use bevy::prelude::*;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .insert_resource(DirectionalLightShadowMap { size: 4096 })
    .add_systems(
        Startup,
        (   
            wrti_test,
            spawn_view_model,
            spawn_lights,
            spawn_text,
            |commands: Commands, asset_server: Res<AssetServer>| setup_glb(commands, asset_server, "cube.glb#Scene0".to_string()),
            // |commands: Commands, asset_server: Res<AssetServer>| setup_glb(commands, asset_server, "calculator.glb#Scene0".to_string()),
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

fn wrti_test() {
    let origin = Vec3::new(0.0, 0.0, 0.0);
    let direction = Vec3::new(0.0, 0.0, 1.0);
    let triangle = (
        Vec3::new(1.0, 0.0, 5.0),
        Vec3::new(-1.0, 1.0, 5.0),
        Vec3::new(-1.0, -1.0, 5.0),
    );
    let backface_culling = false;

    if let Some(hit) = watertight_ray_triangle_intersection(origin, direction, triangle, backface_culling) {
        // call results for testing as needed
        println!("Intersection found at t = {}", hit.t());
        
        // call all the results
        let (t, u, v, w) = hit.as_tuple();
        println!("Hit Breakdown: t: {}, u: {}, v: {}, w: {}", t, u, v, w)
    } else {
        println!("No intersection found");
    }
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
