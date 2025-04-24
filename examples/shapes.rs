use std::path::Path;

use bevy::prelude::*;
use moonshine_core::prelude::*;
use moonshine_view::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Register types for serialization:
        .register_type::<Position>()
        .register_type::<Square>()
        .register_type::<Circle>()
        .register_type::<Special>()
        // Register Shapes as viewable:
        .add_viewable::<Shape>()
        .add_view::<Shape, Square>()
        .add_view::<Shape, Circle>()
        .add_view::<Shape, Special>()
        // Add Save/Load Pipelines:
        .add_systems(
            PreUpdate,
            (
                save_default().into(file_from_resource::<SaveRequest>()),
                load(file_from_resource::<LoadRequest>()),
            ),
        )
        // Gameplay Systems:
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_mouse, handle_keyboard))
        // View Systems:
        .add_systems(Startup, load_assets)
        .add_systems(PostUpdate, view_shape_position_changed)
        .run();
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(Position, Save)]
struct Square;

impl BuildView<Shape> for Square {
    fn build(world: &World, object: Object<Shape>, mut view: ViewCommands<Shape>) {
        info!("{object:?} is observed!");
        let transform: Transform = world.get::<Position>(object.entity()).unwrap().into();
        view.insert((
            transform,
            Gizmo {
                handle: world.resource::<ShapeAssets>().square.clone(),
                ..default()
            },
        ));
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(Position, Save)]
struct Circle;

impl BuildView<Shape> for Circle {
    fn build(world: &World, object: Object<Shape>, mut view: ViewCommands<Shape>) {
        info!("{object:?} is observed!");
        let transform: Transform = world.get::<Position>(object.entity()).unwrap().into();
        view.insert((
            transform,
            Gizmo {
                handle: world.resource::<ShapeAssets>().circle.clone(),
                ..default()
            },
        ));
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(Position, Save)]
struct Special;

impl BuildView<Shape> for Special {
    fn build(world: &World, _object: Object<Shape>, mut view: ViewCommands<Shape>) {
        view.with_children(|view| {
            view.spawn(Gizmo {
                handle: world.resource::<ShapeAssets>().special.clone(),
                ..default()
            });
        });
    }
}

#[derive(Resource)]
struct ShapeAssets {
    square: Handle<GizmoAsset>,
    circle: Handle<GizmoAsset>,
    special: Handle<GizmoAsset>,
}

fn load_assets(mut assets: ResMut<Assets<GizmoAsset>>, mut commands: Commands) {
    // When building views, you cannot mutate the world.
    // This is by design, as it is more efficient to preload the assets you need before building the views.
    let shape_assets = ShapeAssets {
        square: assets.add(square_asset()),
        circle: assets.add(circle_asset()),
        special: assets.add(special_asset()),
    };
    commands.insert_resource(shape_assets);
}

fn square_asset() -> GizmoAsset {
    let mut asset = GizmoAsset::new();
    asset.rect(
        Isometry3d::IDENTITY,
        Vec2::ONE * 10.,
        bevy::color::palettes::css::ORANGE,
    );
    asset
}

fn circle_asset() -> GizmoAsset {
    let mut asset = GizmoAsset::new();
    asset.circle(
        Isometry3d::IDENTITY,
        5.,
        bevy::color::palettes::css::DARK_CYAN,
    );
    asset
}

fn special_asset() -> GizmoAsset {
    let mut asset = GizmoAsset::new();
    asset.circle(Isometry3d::IDENTITY, 8., bevy::color::palettes::css::RED);
    asset
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Position(pub Vec2);

impl Position {
    fn random_in_circle(origin: Vec2, radius: f32) -> Self {
        use rand::Rng;
        let rng = &mut rand::rng();
        let r = rng.random_range(0.0..radius);
        let t = rng.random_range(0.0..2.0 * std::f32::consts::PI);
        let x = r * t.cos();
        let y = r * t.sin();
        let p = Vec2 { x, y };
        Position(origin + p)
    }
}

impl From<&Position> for Transform {
    fn from(&Position(xy): &Position) -> Self {
        Transform::from_translation(xy.extend(0.))
    }
}

struct Shape;

impl Kind for Shape {
    type Filter = (With<Position>, With<Save>);
}

impl BuildView for Shape {
    fn build(_world: &World, _object: Object<Self>, _view: ViewCommands<Self>) {
        // TODO: Base view for all shapes
    }
}

fn setup(mut commands: Commands) {
    const HELP_TEXT: &str = "
    Click Left mouse button to spawn a Square\n
    Click Right mouse button to spawn a Circle\n
    Press 'S' to Save all shapes\n
    Press 'L' to Load all shapes\n
    Press 'R' to Remove all shapes\n
    Press 'M' to Move all shapes to new random positions\n
    Hold 'Ctrl' to spawn a Special shape\n";

    commands.spawn(Camera2d);
    commands.spawn(Text::new(HELP_TEXT));
}

fn handle_mouse(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let position = Position::random_in_circle(Vec2::ZERO, 200.);
        info!("Spawned a Square at {}", position.0);
        let mut shape = commands.spawn((Square, position));
        if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
            shape.insert(Special);
        }
    }
    if mouse.just_pressed(MouseButton::Right) {
        let position = Position::random_in_circle(Vec2::ZERO, 200.);
        info!("Spawned a Circle at {}", position.0);
        let mut shape = commands.spawn((Circle, position));
        if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
            shape.insert(Special);
        }
    }
}

fn handle_keyboard(
    keyboard: Res<ButtonInput<KeyCode>>,
    shapes: Query<Instance<Shape>>,
    positions: Query<&mut Position>,
    mut commands: Commands,
) {
    if keyboard.just_pressed(KeyCode::KeyS) {
        info!("Save!");
        commands.insert_resource(SaveRequest);
    }
    if keyboard.just_pressed(KeyCode::KeyL) {
        info!("Load!");
        commands.insert_resource(LoadRequest);
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        info!("Reset!");
        for shape in shapes.iter() {
            commands.entity(shape.entity()).despawn();
        }
    }
    if keyboard.just_pressed(KeyCode::KeyM) {
        info!("Move!");
        randomize_positions(positions);
    }
}

fn randomize_positions(mut positions: Query<&mut Position>) {
    for mut position in positions.iter_mut() {
        *position = Position::random_in_circle(Vec2::ZERO, 200.);
    }
}

fn view_shape_position_changed(
    shapes: Query<(&Viewable<Shape>, &Position), Changed<Position>>,
    mut transform: Query<&mut Transform>,
) {
    for (viewable, position) in shapes.iter() {
        let view = viewable.view();
        let mut transform = transform.get_mut(view.entity()).unwrap();
        *transform = position.into();
    }
}

#[derive(Resource)]
struct SaveRequest;

impl GetFilePath for SaveRequest {
    fn path(&self) -> &Path {
        Path::new(SAVE_FILE_PATH)
    }
}

#[derive(Resource)]
struct LoadRequest;

impl GetFilePath for LoadRequest {
    fn path(&self) -> &Path {
        Path::new(SAVE_FILE_PATH)
    }
}

const SAVE_FILE_PATH: &str = "shapes.ron";
