use bevy::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use moonshine_core::prelude::*;

use crate::prelude::*;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct M;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct MX;

#[test]
fn test_viewable_spawn() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).register_viewable::<M>();
    app.world_mut().spawn(M);

    app.update();
    assert!(app
        .world_mut()
        .run_system_once(
            |m: Single<Instance<M>, With<Viewable<M>>>, q: Single<&View<M>>| { *m == q.viewable() }
        )
        .unwrap());
}

#[test]
fn test_viewable_despawn() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).register_viewable::<M>();

    // Spawn Viewable
    let m = app.world_mut().spawn(M).id();
    app.update();

    // Get View
    let v = app
        .world_mut()
        .run_system_once(|m: Single<&Viewable<M>>| m.view().entity())
        .unwrap();

    // Despawn Viewable
    app.world_mut()
        .run_system_once(
            |m: Single<Instance<M>, With<Viewable<M>>>, mut commands: Commands| {
                commands.entity(m.entity()).despawn();
            },
        )
        .unwrap();

    app.update();

    assert!(app.world().get_entity(m).is_err());
    assert!(app.world().get_entity(v).is_err());
    assert!(app
        .world_mut()
        .run_system_once(|q: Query<&View<M>>| q.is_empty())
        .unwrap());
}
