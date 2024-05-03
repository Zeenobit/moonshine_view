use std::path::Path;

use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;
use moonshine_kind::prelude::*;
use moonshine_object::prelude::*;
use moonshine_save::prelude::*;
use moonshine_view::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ShapePlugin::default(),
            SavePlugin,
            LoadPlugin,
        ))
        // Register Shape components for Save/Load:
        .register_type::<Square>()
        .register_type::<Circle>()
        .register_type::<Position>()
        // Register Shapes as observale kinds:
        .register_view::<Shape, Square>()
        .register_view::<Shape, Circle>()
        // Add Save/Load Pipelines:
        .add_systems(
            PreUpdate,
            (
                save_default().into_file_on_request::<SaveRequest>(),
                load_from_file_on_request::<LoadRequest>(),
            ),
        )
        // Gameplay Systems:
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_mouse, handle_keyboard))
        // View Systems:
        .add_systems(PostUpdate, observe_shape_position_changed)
        .run();
}

#[derive(Bundle)]
struct SquareBundle {
    square: Square,
    position: Position,
    save: Save,
}

impl SquareBundle {
    fn new(position: Position) -> Self {
        Self {
            square: Square,
            position,
            save: Save,
        }
    }
}

#[derive(Bundle)]
struct CircleBundle {
    circle: Circle,
    position: Position,
    save: Save,
}

impl CircleBundle {
    fn new(position: Position) -> Self {
        Self {
            circle: Circle,
            position,
            save: Save,
        }
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Square;

impl BuildView<Shape> for Square {
    fn build(world: &World, object: Object<Shape>, view: &mut ViewBuilder<Shape>) {
        info!("{object:?} is observed!");
        let transform = world.get::<Position>(object.entity()).unwrap().into();
        view.insert(ShapeBundle::rect(
            &ShapeConfig {
                transform,
                color: Color::ORANGE,
                ..ShapeConfig::default_2d()
            },
            Vec2::ONE * 10.,
        ));
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Circle;

impl BuildView<Shape> for Circle {
    fn build(world: &World, object: Object<Shape>, view: &mut ViewBuilder<Shape>) {
        info!("{object:?} is observed!");
        let transform = world.get::<Position>(object.entity()).unwrap().into();
        view.insert(ShapeBundle::circle(
            &ShapeConfig {
                transform,
                color: Color::CYAN,
                ..ShapeConfig::default_2d()
            },
            5.,
        ));
    }
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Position(pub Vec2);

impl Position {
    fn random_in_circle(origin: Vec2, radius: f32) -> Self {
        use rand::Rng;
        let rng = &mut rand::thread_rng();
        let r = rng.gen_range(0.0..radius);
        let t = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
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

fn setup(mut commands: Commands) {
    const HELP_TEXT: &str = "
    Click Left mouse button to spawn a Square\n
    Click Right mouse button to spawn a Circle\n
    Press 'S' to Save all shapes\n
    Press 'L' to Load all shapes\n
    Press 'R' to Remove all shapes\n
    Press 'M' to Move all shapes to new random positions\n";

    commands.spawn(Camera2dBundle::default());
    commands.spawn(TextBundle {
        text: Text::from_section(HELP_TEXT, TextStyle::default()),
        ..default()
    });
}

fn handle_mouse(input: Res<ButtonInput<MouseButton>>, mut commands: Commands) {
    if input.just_pressed(MouseButton::Left) {
        let position = Position::random_in_circle(Vec2::ZERO, 200.);
        info!("Spawned a Square at {}", position.0);
        commands.spawn(SquareBundle::new(position));
    }
    if input.just_pressed(MouseButton::Right) {
        let position = Position::random_in_circle(Vec2::ZERO, 200.);
        info!("Spawned a Circle at {}", position.0);
        commands.spawn(CircleBundle::new(position));
    }
}

fn handle_keyboard(
    input: Res<ButtonInput<KeyCode>>,
    shapes: Query<Instance<Shape>>,
    positions: Query<&mut Position>,
    mut commands: Commands,
) {
    if input.just_pressed(KeyCode::KeyS) {
        info!("Save!");
        commands.insert_resource(SaveRequest);
    }
    if input.just_pressed(KeyCode::KeyL) {
        info!("Load!");
        commands.insert_resource(LoadRequest);
    }
    if input.just_pressed(KeyCode::KeyR) {
        info!("Reset!");
        for shape in shapes.iter() {
            commands.entity(shape.entity()).despawn_recursive();
        }
    }
    if input.just_pressed(KeyCode::KeyM) {
        info!("Move!");
        randomize_positions(positions);
    }
}

fn randomize_positions(mut positions: Query<&mut Position>) {
    for mut position in positions.iter_mut() {
        *position = Position::random_in_circle(Vec2::ZERO, 200.);
    }
}

fn observe_shape_position_changed(
    shapes: Query<(&Model<Shape>, &Position), Changed<Position>>,
    mut transform: Query<&mut Transform>,
) {
    for (observer, position) in shapes.iter() {
        let view = observer.view();
        let mut transform = transform.get_mut(view.entity()).unwrap();
        *transform = position.into();
    }
}

#[derive(Resource)]
struct SaveRequest;

impl SaveIntoFileRequest for SaveRequest {
    fn path(&self) -> &Path {
        Path::new("shapes.ron")
    }
}

#[derive(Resource)]
struct LoadRequest;

impl LoadFromFileRequest for LoadRequest {
    fn path(&self) -> &Path {
        Path::new("shapes.ron")
    }
}
