use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
    window::{PrimaryWindow, WindowResized},
};
use sickle_ui::ui_commands::UpdateStatesExt;

use crate::{GameState, SkipRemove};

/// In-game resolution width.
pub const RES_WIDTH: u32 = 240;

/// In-game resolution height.
pub const RES_HEIGHT: u32 = 135;

/// Default render layers for pixel-perfect rendering.
/// You can skip adding this component, as this is the default.
pub const PIXEL_PERFECT_LAYER: RenderLayers = RenderLayers::layer(0);

/// Render layers for high-resolution rendering.
pub const HIGH_RES_LAYER: RenderLayers = RenderLayers::layer(1);

pub struct PixelPerfectPlugin;

impl Plugin for PixelPerfectPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa::Off)
            .add_systems(Startup, setup_camera)
            .add_systems(Update, fit_canvas);
    }
}

/// Low-resolution texture that contains the pixel-perfect world.
/// Canvas itself is rendered to the high-resolution world.
#[derive(Component)]
struct Canvas;

/// Camera that renders the pixel-perfect world to the [`Canvas`].
#[derive(Component)]
pub struct InGameCamera;

/// Camera that renders the [`Canvas`] (and other graphics on [`HIGH_RES_LAYERS`]) to the screen.
#[derive(Component)]
pub struct OuterCamera;

fn setup_camera(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let canvas_size = Extent3d {
        width: RES_WIDTH,
        height: RES_HEIGHT,
        ..default()
    };

    // this Image serves as a canvas representing the low-resolution game screen
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    canvas.resize(canvas_size);

    let image_handle = images.add(canvas);

    // this camera renders whatever is on `PIXEL_PERFECT_LAYERS` to the canvas
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                // render before the "main pass" camera
                order: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..default()
            },
            // projection: OrthographicProjection {
            //     scale: 1.0 / (2. as f32),
            //     ..default()
            // },
            ..default()
        },
        SkipRemove,
        InGameCamera,
        PIXEL_PERFECT_LAYER,
    ));

    // spawn the canvas
    commands.spawn((
        SpriteBundle {
            texture: image_handle,
            transform: Transform::from_xyz(0., 0., -999.),
            ..default()
        },
        SkipRemove,
        Canvas,
        HIGH_RES_LAYER,
    ));

    // the "outer" camera renders whatever is on `HIGH_RES_LAYERS` to the screen.
    // here, the canvas and one of the sample sprites will be rendered by this camera
    commands.spawn((
        Camera2dBundle {
            // projection: OrthographicProjection {
            //     scale: 1.0 / (2. as f32),
            //     ..default()
            // },
            ..Default::default()
        },
        SkipRemove,
        OuterCamera,
        HIGH_RES_LAYER,
    ));
}

/// Scales camera projection to fit the window (integer multiples only).
fn fit_canvas(
    mut resize_events: EventReader<WindowResized>,
    mut projections: Query<&mut OrthographicProjection, With<OuterCamera>>,
    mut ui_scale: ResMut<UiScale>,
) {
    for event in resize_events.read() {
        let h_scale = event.width / RES_WIDTH as f32;
        let v_scale = event.height / RES_HEIGHT as f32;
        let mut projection = projections.single_mut();
        projection.scale = 1. / h_scale.min(v_scale);
        ui_scale.0 = h_scale.min(v_scale) * (RES_WIDTH as f32 / 1920.);
    }
}
