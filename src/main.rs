use bevy::{ecs::query::QueryData, math, prelude::*, render::Render, sprite::Mesh2dHandle};

#[derive(Component, QueryData)]
struct Player;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, start)
        .add_systems(Render, render)
        .add_systems(Update, move_player)
        .run();
}

fn start(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let color = Color::rgb(1.0, 0.0, 0.0);
    let mat = materials.add(color);

    let shape = Mesh2dHandle(meshes.add(Circle { radius: 32.0 }));
    commands
        .spawn(ColorMesh2dBundle {
            mesh: shape,
            material: mat.clone(),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        })
        .insert(Player);

    let test = Mesh2dHandle(meshes.add(Circle { radius: 16.0 }));
    commands.spawn(ColorMesh2dBundle {
        mesh: test,
        material: mat.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    {
        let pl = PointLight {
            color: (Color::rgb(1.0, 0.5, 0.8)),
            intensity: (1.0),
            range: (32.0),
            radius: (128.0),
            shadows_enabled: (true),
            shadow_depth_bias: (1.0),
            shadow_normal_bias: (1.0),
        };

        let mut ent = commands.spawn_empty();
        ent.insert(pl);

        let tx = Transform {
            translation: Vec3::ONE,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };

        ent.insert(tx);
    }
}

fn render(mut commands: Commands) {}

fn move_player(
    mut commands: Commands,
    mut query: Query<&mut Transform, With<Player>>,
    mut cam_query: Query<
        (&mut Transform, &mut OrthographicProjection),
        (With<Camera2d>, Without<Player>),
    >,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let mv_speed = 325.0 * dt;

    let mut player_xform = query.single_mut();
    let (mut cam_xform, mut cam_proj) = cam_query.single_mut();

    if keyboard_input.pressed(KeyCode::KeyW) {
        player_xform.translation.y += mv_speed;
    }

    if keyboard_input.pressed(KeyCode::KeyS) {
        player_xform.translation.y -= mv_speed;
    }

    if keyboard_input.pressed(KeyCode::KeyD) {
        player_xform.translation.x += mv_speed;
    }

    if keyboard_input.pressed(KeyCode::KeyA) {
        player_xform.translation.x -= mv_speed;
    }

    cam_xform.translation = player_xform.translation;

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        cam_proj.scale -= 1.2 * dt;
    } else if keyboard_input.pressed(KeyCode::ArrowDown) {
        cam_proj.scale += 1.2 * dt;
    }

    cam_proj.scale = f32::clamp(cam_proj.scale, 0.5, 1.5)
}
