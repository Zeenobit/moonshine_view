# 👁️ Moonshine View

A generic solution for separating the game view from game state designed for [Moonshine Save](https://github.com/Zeenobit/moonshine_save) framework.

## Overview

The Moonshine Save system is intentionally designed to encourage the user to separate the persistent game state (model) from its aesthetic elements (view). This provides a clear separation of concerns and has various benefits which are explained in detail in the [save framework](https://github.com/Zeenobit/moonshine_save#Philosophy) documentation.

An issue with this approach is that it adds additional complexity that the developer has to maintain. Typically, this involves manually de/spawning views associated with saved entities and synchronizing them with the game state via systems.

This crate aims to reduce some of this complexity by providing a more generic and ergonomic solution for synchronizing the game view with the game state.

## Usage

### Observables

By definition, an observable is any [`Kind`](https://docs.rs/moonshine-kind/latest/moonshine_kind/trait.Kind.html) which implements the `Observe` trait. Typically, this is any entity which has an observable representation in your game.

```rust
use bevy::prelude::*;
use moonshine_view::prelude::*;

#[derive(Component)]
struct Bird;

impl Observe for Bird {
    fn observe(world: &World, object: Object<Self>, view: &mut ViewBuilder<Self>) {
        let asset_server = world.resource::<AssetServer>();
        // ...
    }
}
```

You must register your type as an observable when building your [`App`]:

```rust,ignore
app.register_observable::<Bird>();
```

Note that you can define any kind as observable, not just components!

For example:

```rust,ignore
struct Creature;

impl Kind for Creature {
    type Filter = Or<(With<Monkey>, With<Bird>)>;
}

impl Observe for Creature {
    fn observe(world: &World, object: Object<Self>, view: &mut ViewBuilder<Self>) {
        // ...
    }
}
```

### Observers and Views

Whenever an observable instance of kind `T` is found without an `Observer<T>` component, a new view is spawned and observable is invoked by calling `Observe::observe`.

This happens automatically. Any entity with an `Observer<T>` component is associated with a `View<T>`, which can be accessed via the `Observer<T>::view` method. Conversely, any entity with a `View<T>` component is associated with an `Observer<T>`, which can be accessed via the `View<T>::target` method.

Together, `Observer<T>` and `View<T>` form a bidirectional link between the game state and the game view.

These can be used to synchronize the view from the game state (push) or query the game state from the view (pull).

The "push" approach should be preferred because it often leads to less iterations per update cycle.

```rust,ignore
fn observe_bird_moved(query: Query<(&Bird, &Observer<Bird>), Changed<Position>>) {
    for (bird, observer) in query.iter() {
        let view = observer.view();
        // TODO: Update the view
    }
}
```

Alternatively, you may also "pull" the game state into the view by querying the view target:

```rust,ignore
fn view_bird(views: Query<&View<Bird>>, query: Query<&Bird, Changed<Position>>) {
    for view in views.iter() {
        if let Ok(bird) = query.get(view.target().entity()) {
            // TODO: Update the view from bird
        }
    }
}
```

### View Builder

When `Observe::observe` is invoked by the view system, an entity with a `View<T>` component is created and passed to the `ViewBuilder<T>`. You may use the `ViewBuilder<T>` to either:
1. Insert components into the root view entity.
2. Spawn children attached to the root view entity.

```rust,ignore
#[derive(Bundle)]
struct BirdViewBundle {
    // ...
}

#[derive(Bundle)]
struct BirdWingsViewBundle {
    // ...
}

impl Observe for Bird {
    fn observe(world: &World, object: Object<Self>, view: &mut ViewBuilder<Self>) {
        // You have immutable access to the world and the observable entity's hierarchy.
        // Build the view as needed!

        let asset_server = world.resource::<AssetServer>();
        view.insert(BirdViewBundle { /* ... */ });
        view.insert_children(|root| {
            root.spawn(BirdWingsViewBundle { /* ... */ });
            // ...
        });
    }
}
```

The root view entity is automatically marked with an [`Unload`](https://docs.rs/moonshine-save/latest/moonshine_save/load/struct.Unload.html) component. This means that the entire view entity hierarchy will be automatically despawned whenever a new game state is loaded.

The view entity hierarchy will also despawn automatically whenever its target observable is not longer valid (i.e. if despawned or no longer matching the correct kind).

### Generic Observables

Because the view system uses kinds to ensure full type safety between views and observables, there is no way to access all views of a given observable entity via a component.

Instead, you may query all views associated with an entity by using the `Observables` resource:

```rust,ignore
fn update_views_generic(observables: Res<Observables>) {
    for observable_entity in observables.iter() {
        for view_entity in observables.views(observable_entity) {
            // ...
        }
    }
}
```

## Examples

See [shapes.rs](examples/shapes.rs) for a complete usage example.