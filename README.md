# 👁️ Moonshine View

A generic solution for separating the game view from game logic specifically designed for [Moonshine Save](https://github.com/Zeenobit/moonshine_save) framework.

## Overview

The Moonshine Save system is intentionally designed to encourage the user to separate the persistent game state from its aesthetic elements. This provides a clear separation of concerns which which has various benefits explained in detail in the [save framework](https://github.com/Zeenobit/moonshine_save#Philosophy) documentation.

The main drawback of this approach is that it adds additional boilerplate that the user has to deal with. Typically, the user has to manually spawn/despawn views associated with saved entities and update them accordingly at runtime.

This crate aims to solve this issue by providing a more generic and ergonomic solution for synchronizing the game view with the game state.

## Usage

### Observables

By definition, an observable is any [`Kind`](https://docs.rs/moonshine-kind/latest/moonshine_kind/trait.Kind.html) which implements the `Observe` trait:

```rust
use bevy::prelude::*;
use moonshine_view::prelude::*;

#[derive(Component)]
struct Bird;

impl Observe for Bird {
    fn observe(world: &World, object: Object<Self>, view: &mut ViewBuilder<Self>) {
        let asset_server = world.resource::<AssetServer>();
        // TODO: Build the Bird view
    }
}
```

You may register your type as an observable when building your `App`:

```rust
app.add_plugin(ViewPlugin) // Add `ViewPlugin` before registering observables!
    .register_observable::<Square>();
```

Note that you can define any kind as observable, and not just components!

For example:

```rust
use bevy::prelude::*;
use moonshine_kind::prelude::*;
use moonshine_view::prelude::*;

struct Creature {
    type Filter = Or<(With<Monkey>, With<Bird>)>;
}

impl Observe for Creature {
    ...
}
```

### Observers and Views

Whenever an observable instance of kind `T` is found without an `Observer<T>` component, a new view is spawned and observable is invoked by calling `Observe::observe`.

This happens automatically. Any entity with an `Observer<T>` component is associated with a `View<T>`, which can be accessed via the `Observer<T>::view` method. Conversely, any entity with a `View<T>` component is associated with an `Observer<T>`, which can be accessed via the `View<T>::target` method.

Together, `Observer<T>` and `View<T>` form a bidirectional link between the game state and the game view.

These can be used to synchronize the view from the game state (push) or query the game state from the view (pull):

```rust
use moonshine_kind::prelude::*;

fn push_bird_state(query: Query<(&Bird, &Observer<Bird>), Changed<Position>>) {
    for (bird, observer) in query.iter() {
        let view: Instance<View<T>> = observer.view();
        // TODO: Update the view
    }
}

fn pull_bird_state(views: Query<&View<Bird>>, query: Query<&Bird, Changed<Position>>) {
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

```rust
impl Observe for Bird {
    fn observe(world: &World, object: Object<Self>, view: &mut ViewBuilder<Self>) {
        // You have immutable access to the world and the observable entity's hierarchy.
        // Build the view as needed!

        let asset_server = world.resource::<AssetServer>();
        view.insert(...);
        view.spawn(|root| {
            root.spawn(...);
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

```rust
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