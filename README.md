# 👁️ Moonshine View

[![crates.io](https://img.shields.io/crates/v/moonshine-view)](https://crates.io/crates/moonshine-view)
[![downloads](https://img.shields.io/crates/dr/moonshine-view?label=downloads)](https://crates.io/crates/moonshine-view)
[![docs.rs](https://docs.rs/moonshine-save/badge.svg)](https://docs.rs/moonshine-view)
[![license](https://img.shields.io/crates/l/moonshine-view)](https://github.com/Zeenobit/moonshine_view/blob/main/LICENSE)
[![stars](https://img.shields.io/github/stars/Zeenobit/moonshine_view)](https://github.com/Zeenobit/moonshine_view)

Generic [Model/View](https://en.wikipedia.org/wiki/Model%E2%80%93view%E2%80%93controller) framework designed for [Bevy](https://bevyengine.org/) and the [Moonshine](https://github.com/Zeenobit/moonshine_core) save system.

## Overview

The Moonshine Save system is intentionally designed to encourage the user to separate the persistent game state (model) from its aesthetic elements (view). This provides a clear separation of concerns and has various benefits which are explained in detail in the [save framework](https://github.com/Zeenobit/moonshine_save#Philosophy) documentation.

An issue with this approach is that it adds additional complexity that the developer has to maintain. Typically, this involves manually de/spawning views associated with saved entities and synchronizing them with the game state via systems.

This crate aims to reduce some of this complexity by providing a more generic and ergonomic solution for synchronizing the game view with the game state.

## Usage

### Viewables

By definition, a [`Component`] is **Viewable** if a view can be built for it using [`BuildView`].

An [`Entity`] is **Viewable** if it matches a registered [`ViewableKind`].

```rust
use bevy::prelude::*;
use moonshine_core::prelude::*;
use moonshine_view::prelude::*;

#[derive(Component)]
struct Bird;

impl ViewableKind for Bird {}

fn build_bird_view(trigger: Trigger<OnBuildView<Bird>>, mut commands: Commands) {
    let viewable = trigger.target();
    let view = trigger.view();
    commands.entity(*view).insert(BirdView);
}

#[derive(Component)]
struct BirdView;

// Remember to register viewable kinds:
let mut app = App::new();
app.register_viewable::<Bird>();
app.world_mut().spawn(Bird);
```

> [!WARNING]
> Order of operations is undefined when multiple views are built for the same entity kind.</br>
> Prefer to add components/children when building views to avoid ordering issues.

### Viewable ⇄ View

When a viewable entity is spawned, a **View Entity** is spawned with it.

A view entity is an entity with at least one [`View<T>`] component. Each [`View<T>`] is associated with its model entity via [`Viewable<T>`].

When a [`Viewable<T>`] is despawned, or if it is no longer of [`Kind`] `T`, the associated view entity is despawned with it.

Together, [`Viewable<T>`] and [`View<T>`] form a two-way link between the game state and the game view.

### Synchronization

At runtime, it is often required to update the view state based on the viewable state. For example, if an entity's position changes, so should the position of its view.

To solve this, consider using a system which either updates the view based on latest viewable state ("push") or queries the viewable from the view ("pull").

The "push" approach should be preferred because it often leads to less iterations per update cycle.

```rust
use bevy::prelude::*;
use moonshine_core::prelude::*;
use moonshine_view::prelude::*;

#[derive(Component)]
struct Bird;

impl ViewableKind for Bird {}

// Update view from viewable, if needed (preferred)
fn view_bird_changed(query: Query<(&Bird, &Viewable<Bird>), Changed<Bird>>) {
    for (bird, model) in query.iter() {
        let view = model.view();
        // ...
    }
}

// Query model from view constantly (typically less efficient)
fn view_bird(views: Query<&View<Bird>>, query: Query<&Bird, Changed<Bird>>) {
    for view in views.iter() {
        let viewable = view.viewable();
        if let Ok(bird) = query.get(viewable.entity()) {
            // ...
        }
    }
}
```

The root view entity is automatically marked with [`Unload`].

This means the entire view entity hierarchy is despawned whenever a new game state is loaded.

## Examples

See [shapes.rs](examples/shapes.rs) for a complete usage example.

[`Component`]:https://docs.rs/bevy/latest/bevy/ecs/component/trait.Component.html
[`Entity`]:https://docs.rs/bevy/latest/bevy/ecs/entity/struct.Entity.html
[`Kind`]:https://docs.rs/moonshine-kind/latest/moonshine_kind/trait.Kind.html
[`Unload`]:https://docs.rs/moonshine-save/latest/moonshine_save/load/struct.Unload.html
[`ViewableKind`]:https://docs.rs/moonshine-view/latest/moonshine_view/trait.ViewableKind.html
[`Viewable<T>`]:https://docs.rs/moonshine-view/latest/moonshine_view/struct.Viewable.html
[`View<T>`]:https://docs.rs/moonshine-view/latest/moonshine_view/struct.View.html
[`RegisterView`]:https://docs.rs/moonshine-view/latest/moonshine_view/trait.RegisterView.html
[`ViewCommands`]:https://docs.rs/moonshine-view/latest/moonshine_view/struct.ViewCommands.html

## Support

Please [post an issue](https://github.com/Zeenobit/moonshine_view/issues/new) for any bugs, questions, or suggestions.

You may also contact me on the official [Bevy Discord](https://discord.gg/bevy) server as **@Zeenobit**.
