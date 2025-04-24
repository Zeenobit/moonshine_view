use bevy::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use moonshine_core::prelude::*;

use crate::prelude::*;

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct M;

impl BuildView for M {
    fn build(_world: &World, _object: Object<Self>, _view: ViewCommands<Self>) {}
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct MX;

impl BuildView<M> for MX {
    fn build(_world: &World, _object: Object<M>, _view: ViewCommands<M>) {}
}

#[test]
fn test_viewable_spawn() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_viewable::<M>();
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
    app.add_plugins(MinimalPlugins).add_viewable::<M>();
    app.world_mut().spawn(M);

    app.update();
    app.world_mut()
        .run_system_once(
            |m: Single<Instance<M>, With<Viewable<M>>>, mut commands: Commands| {
                commands.entity(m.entity()).despawn();
            },
        )
        .unwrap();

    app.update();
    assert!(app
        .world_mut()
        .run_system_once(|q: Query<&View<M>>| q.is_empty())
        .unwrap());
}
