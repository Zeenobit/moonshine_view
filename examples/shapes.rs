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
        .register_viewable::<Shape>()
        // Add observers to build views:
        .add_observer(build_shape_view)
        .add_observer(build_square_view)
        .add_observer(build_circle_view)
        .add_observer(build_special_view)
        // Add Save/Load Observers:
        .add_observer(save_on_default_event)
        .add_observer(load_on_default_event)
        // Gameplay Systems:
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_mouse, handle_keyboard))
        // View Systems:
        .add_systems(Startup, load_assets)
        .add_systems(PostUpdate, handle_shape_position_changed)
        .run();
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(Position, Save)]
struct Square;

fn build_square_view(
    trigger: Trigger<OnAdd, Viewable<Shape>>,
    query: Query<(Instance<Square>, &Viewable<Shape>)>,
    assets: Res<ShapeAssets>,
    mut commands: Commands,
) {
    let Ok((instance, viewable)) = query.get(trigger.target()) else {
        // Target is not a Square
        return;
    };

    info!("{instance} is observed!");

    let view = viewable.view();
    commands.instance(view).with_child(Gizmo {
        handle: assets.square.clone(),
        ..default()
    });
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(Position, Save)]
struct Circle;

fn build_circle_view(
    trigger: Trigger<OnAdd, Viewable<Shape>>,
    query: Query<(Instance<Circle>, &Viewable<Shape>)>,
    assets: Res<ShapeAssets>,
    mut commands: Commands,
) {
    let Ok((instance, viewable)) = query.get(trigger.target()) else {
        // Target is not a Circle
        return;
    };

    info!("{instance} is observed!");

    commands.instance(viewable.view()).with_child(Gizmo {
        handle: assets.circle.clone(),
        ..default()
    });
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
#[require(Position, Save)]
struct Special;

fn build_special_view(
    trigger: Trigger<OnAdd, Viewable<Shape>>,
    query: Query<(Instance<Special>, &Viewable<Shape>)>,
    assets: Res<ShapeAssets>,
    mut commands: Commands,
) {
    let Ok((instance, viewable)) = query.get(trigger.target()) else {
        // Target is not a Special shape
        return;
    };

    info!("{instance} is observed!");

    commands.instance(viewable.view()).with_child(Gizmo {
        handle: assets.special.clone(),
        ..default()
    });
}

#[derive(Resource)]
struct ShapeAssets {
    base: Handle<GizmoAsset>,
    square: Handle<GizmoAsset>,
    circle: Handle<GizmoAsset>,
    special: Handle<GizmoAsset>,
}

fn load_assets(mut assets: ResMut<Assets<GizmoAsset>>, mut commands: Commands) {
    // You could load these assets directly when building views.
    // However, it is often more efficient to load them just once at startup.
    let shape_assets = ShapeAssets {
        base: assets.add(base_asset()),
        square: assets.add(square_asset()),
        circle: assets.add(circle_asset()),
        special: assets.add(special_asset()),
    };
    commands.insert_resource(shape_assets);
}

fn base_asset() -> GizmoAsset {
    let mut asset = GizmoAsset::new();
    asset.circle(Isometry3d::IDENTITY, 1., bevy::color::palettes::css::WHITE);
    asset
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

#[derive(Component, Default, Debug, Reflect)]
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

impl ViewableKind for Shape {
    fn view_bundle() -> impl Bundle {
        (Unload, GlobalTransform::default(), Transform::default())
    }
}

fn build_shape_view(
    trigger: Trigger<OnAdd, Viewable<Shape>>,
    query: Query<(Instance<Shape>, &Position, &Viewable<Shape>)>,
    assets: Res<ShapeAssets>,
    mut commands: Commands,
) {
    let (instance, position, viewable) = query.get(trigger.target()).unwrap();
    info!("{instance} is observed!");
    let transform: Transform = position.into();
    commands.instance(viewable.view()).insert((
        transform,
        Gizmo {
            handle: assets.base.clone(),
            ..default()
        },
    ));
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
        commands.trigger_save(SaveWorld::default_into_file(SAVE_FILE_PATH));
    }
    if keyboard.just_pressed(KeyCode::KeyL) {
        info!("Load!");
        commands.trigger_load(LoadWorld::default_from_file(SAVE_FILE_PATH));
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

fn handle_shape_position_changed(
    shapes: Query<(&Viewable<Shape>, &Position), Changed<Position>>,
    mut transform: Query<&mut Transform>,
) {
    for (viewable, position) in shapes.iter() {
        let view = viewable.view();
        let mut transform = transform.get_mut(view.entity()).unwrap();
        *transform = position.into();
    }
}

const SAVE_FILE_PATH: &str = "shapes.ron";
