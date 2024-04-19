use bevy::prelude::*;

#[derive(Component)]
pub struct LowResCamera;

#[derive(Component)]
pub struct MainCamera;

pub(crate) fn init(mut app: &mut App) {
    // app.add_event::<SetCameraModeEvent>();
    app.add_systems(Update, camera_state_handler);
}

#[derive(Resource)]
pub(crate) struct CameraState {
    /// If scene_params is None, just be a FPS camera on the player.
    pub scene_params: Option<CameraSceneParams>,
    pub elapsed_time: f32,
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            scene_params: None,
            elapsed_time: 0.0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct CameraSceneParams {
    // pub entity: Entity,
    pub target_position: Vec3,
    pub pos_offset: Vec3,
    // pub rotation: Quat,
    pub duration: f32,
}

pub(crate) fn camera_state_handler(
    mut query: Query<&mut Transform, With<LowResCamera>>,
    mut camera_state: ResMut<CameraState>,
    time: Res<Time>,
) {
    // Let the player system handle the camera
    if camera_state.scene_params.is_none() {
        return;
    }

    let mut cam_xform = query.single_mut();
    let dt = time.delta_seconds();

    if let Some(info) = camera_state.scene_params {
        cam_xform.translate_around(
            info.target_position,
            Quat::from_axis_angle(Vec3::Y, 0.6 * dt),
        );
        cam_xform.look_at(info.target_position, Vec3::Y);
        camera_state.elapsed_time += dt;

        if camera_state.elapsed_time >= info.duration {
            camera_state.scene_params = None;
            camera_state.elapsed_time = 0.0;
        }
    }
}

// #[derive(Eq, PartialEq)]
// pub(crate) enum CameraMode {
//     PlayerView,
//     Scene(CameraSceneParams),
// }

// #[derive(Component)]
// pub(crate) struct SceneCameraTarget;

// #[derive(Event)]
// pub(crate) struct SetCameraModeEvent {
//     pub next_mode: CameraMode,
//     pub duration: Option<f32>,
// }

// fn set_camera_mode_listener(mut camera_state: ResMut<CameraState>) {
//     for ev in events.read() {
//         match ev.next_mode {
//             CameraMode::PlayerView => camera_state.mode = CameraMode::PlayerView,
//             CameraMode::Scene(scene_params) => {
//                 camera_state.mode = CameraMode::Scene(scene_params);
//             }
//         }
//     }
// }
