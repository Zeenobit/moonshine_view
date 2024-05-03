# 👁️ Moonshine View

A generic solution for separating the game view from game state designed for [Moonshine Save](https://github.com/Zeenobit/moonshine_save) framework.

## Overview

The Moonshine Save system is intentionally designed to encourage the user to separate the persistent game state (model) from its aesthetic elements (view). This provides a clear separation of concerns and has various benefits which are explained in detail in the [save framework](https://github.com/Zeenobit/moonshine_save#Philosophy) documentation.

An issue with this approach is that it adds additional complexity that the developer has to maintain. Typically, this involves manually de/spawning views associated with saved entities and synchronizing them with the game state via systems.

This crate aims to reduce some of this complexity by providing a more generic and ergonomic solution for synchronizing the game view with the game state.

## Usage

### Viewables

By definition, a [`Component`] is **Viewable** if a view can be built for it using [`BuildView`].

An [`Entity`] is **Viewable** if it has at least one viewable component.

```rust
use bevy::prelude::*;
use moonshine_view::prelude::*;

#[derive(Component)]
struct Bird;

impl BuildView for Bird {
    fn build(world: &World, object: Object<Self>, view: &mut ViewBuilder<Self>) {
        let asset_server = world.resource::<AssetServer>();
        // ...
        for child in object.children() {
            // ...
        }
    }
}

// Remember to register viewable types:
let mut app = App::new();
app.register_viewable::<Bird>();
```

You may also define a [`Kind`] as viewable:

```rust
use bevy::prelude::*;
use moonshine_view::prelude::*;
use moonshine_kind::prelude::*;

#[derive(Component)]
struct Bird;

#[derive(Component)]
struct Monkey;

struct Creature;

impl Kind for Creature {
    type Filter = Or<(With<Bird>, With<Monkey>)>;
}

impl BuildView for Creature {
    fn build(world: &World, object: Object<Self>, view: &mut ViewBuilder<Self>) {
        // All creatures look the same!
    }
}

// Remember to register viewable types:
let mut app = App::new();
app.register_viewable::<Creature>();
```

This is useful when you want to define the same view for multiple kinds of entities.

You may even define views polymorphically.

```rust
use bevy::prelude::*;
use moonshine_view::prelude::*;
use moonshine_kind::prelude::*;

#[derive(Component)]
struct Bird;

#[derive(Component)]
struct Monkey;

struct Creature;

impl Kind for Creature {
    type Filter = Or<(With<Bird>, With<Monkey>)>;
}

impl BuildView<Creature> for Bird {
    fn build(world: &World, object: Object<Creature>, view: &mut ViewBuilder<Creature>) {
        // Birds look different, but they're still creatures!
    }
}

impl BuildView<Creature> for Monkey {
    fn build(world: &World, object: Object<Creature>, view: &mut ViewBuilder<Creature>) {
        // Monkeys look different, but they're still creatures!
    }
}

// Polymorphic views are registered slightly differently:
let mut app = App::new();
app.register_view::<Creature, Bird>()
    .register_view::<Creature, Monkey>();
```

This is useful when you want to build a different version of the same view for multiple kinds of entities.

### Model ⇄ View

When a viewable entity is spawned, a **View Entity** is spawned with it, and the viewable entity becomes a **Model Entity**.

A view entity is an entity with at least one [`View<T>`] component. Similarly, a model entity is an entity with at least one [`Model<T>`] component.

Each [`View<T>`] is associated with exactly one [`Model<T>`].

When a [`Model<T>`] is despawned, or if it is no longer of [`Kind`] `T`, the associated [`View<T>`] is despawned with it.

Assuming the view is registered (see [`RegisterView`]), all of this happens automatically. 😎

Together, [`Model<T>`] and [`View<T>`] form a two-way link between the game state and the game view.

### Synchronization

At runtime, it is often required to update the view state based on the model state. For example, if an entity's position changes, so should the position of its view.

To solve this, consider using a system which either updates the view based on latest model state ("push") or queries the model from the view ("pull").

The "push" approach should be preferred because it often leads to less iterations per update cycle.

```rust
use bevy::prelude::*;
use moonshine_view::prelude::*;

#[derive(Component)]
struct Bird;

impl BuildView for Bird {
    fn build(world: &World, object: Object<Self>, view: &mut ViewBuilder<Self>) {
        // ...
    }
}

// Update view from model, if needed (preferred)
fn view_bird_moved(query: Query<(&Bird, &Model<Bird>), Changed<Bird>>) {
    for (bird, model) in query.iter() {
        let view = model.view();
        // ...
    }
}

// Query model from view constantly (typically less efficient)
fn view_bird(views: Query<&View<Bird>>, query: Query<&Bird, Changed<Bird>>) {
    for view in views.iter() {
        let model = view.model();
        if let Ok(bird) = query.get(model.entity()) {
            // ...
        }
    }
}
```

### View Builder

The implementation of [`BuildView`] requires the use [`ViewBuilder`] to setup the view entity as needed.

The view builder may be used to:
1. Insert components into the view entity, or
2. Add new children to the view entity

For example,

```rust
use bevy::prelude::*;
use moonshine_view::prelude::*;

#[derive(Component)]
struct Bird;

#[derive(Bundle)]
struct BirdViewBundle {
    // ...
}

#[derive(Bundle)]
struct BirdWingsViewBundle {
    // ...
}

impl BuildView for Bird {
    fn build(world: &World, object: Object<Self>, view: &mut ViewBuilder<Self>) {
        // Root
        view.insert(BirdViewBundle {
            // ...
        });

        view.insert_children(|root| {
            // Wings
            root.spawn(BirdWingsViewBundle {
                // ...
            });
        });
    }
}
```

The root view entity is automatically marked with [`Unload`].

This means the entire view entity hierarchy is despawned whenever a new game state is loaded.

### Untyped Viewables

Because the view system uses [`Kind`] for type safety, there is no access to views of a given viewable entity via a component.

Instead, you may query all views associated with an entity by using the `Observables` resource:

```rust
use bevy::prelude::*;
use moonshine_view::prelude::*;

fn update_views_generic(viewables: Res<Viewables>) {
    for viewable_entity in viewables.iter() {
        for view_entity in viewables.views(viewable_entity) {
            // ...
        }
    }
}
```

## Examples

See [shapes.rs](examples/shapes.rs) for a complete usage example.

[`Component`]:https://docs.rs/bevy/latest/bevy/ecs/component/trait.Component.html
[`Entity`]:https://docs.rs/bevy/latest/bevy/ecs/entity/struct.Entity.html
[`Kind`]:https://docs.rs/moonshine-kind/latest/moonshine_kind/trait.Kind.html
[`Unload`]:https://docs.rs/moonshine-save/latest/moonshine_save/load/struct.Unload.html
[`BuildView`]:https://docs.rs/moonshine-view/latest/moonshine_view/trait.Observe.html
[`Model<T>`]:https://docs.rs/moonshine-view/latest/moonshine_view/struct.Model.html
[`View<T>`]:https://docs.rs/moonshine-view/latest/moonshine_view/struct.View.html
[`RegisterView`]:https://docs.rs/moonshine-view/latest/moonshine_view/trait.RegisterView.html
[`ViewBuilder`]:https://docs.rs/moonshine-view/latest/moonshine_view/struct.ViewBuilder.html