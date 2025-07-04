#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::relationship::Relationship;
use moonshine_kind::prelude::*;

pub mod prelude {
    pub use super::{OnBuildView, RegisterViewable, View, ViewSystems, Viewable};
}

#[cfg(test)]
mod tests;

pub trait RegisterViewable {
    /// Adds a given [`Kind`] as viewable.
    fn register_viewable<T: Kind>(&mut self) -> &mut Self;
}

impl RegisterViewable for App {
    /// Adds a given [`Kind`] as viewable.
    fn register_viewable<T: Kind>(&mut self) -> &mut Self {
        self.add_systems(PreUpdate, trigger_build_view::<T>.in_set(ViewSystems));
        self
    }
}

#[derive(SystemSet, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ViewSystems;

/// [`Component`] of an [`Entity`] associated with a [`Viewable`].
#[derive(Component)]
#[component(on_insert = <Self as Relationship>::on_insert)]
#[component(on_replace = <Self as Relationship>::on_replace)]
pub struct View<T: Kind> {
    viewable: Instance<T>,
}

impl<T: Kind> View<T> {
    /// Returns the associated viewable entity.
    pub fn viewable(&self) -> Instance<T> {
        self.viewable
    }
}

impl<T: Kind> Relationship for View<T> {
    type RelationshipTarget = Viewable<T>;

    fn get(&self) -> Entity {
        self.viewable.entity()
    }

    fn from(entity: Entity) -> Self {
        Self {
            viewable: unsafe { Instance::from_entity_unchecked(entity) },
        }
    }
}

#[derive(Component, Debug)]
#[component(on_replace = <Self as RelationshipTarget>::on_replace)]
#[component(on_despawn = <Self as RelationshipTarget>::on_despawn)]
pub struct Viewable<T: Kind> {
    view: Instance<View<T>>,
}

impl<T: Kind> Viewable<T> {
    /// Returns the [`View`] [`Instance`] associated with this [`Viewable`].
    pub fn view(&self) -> Instance<View<T>> {
        self.view
    }
}

impl<T: Kind> RelationshipTarget for Viewable<T> {
    const LINKED_SPAWN: bool = true;

    type Relationship = View<T>;

    type Collection = Instance<View<T>>;

    fn collection(&self) -> &Self::Collection {
        &self.view
    }

    fn collection_mut_risky(&mut self) -> &mut Self::Collection {
        &mut self.view
    }

    fn from_collection_risky(collection: Self::Collection) -> Self {
        Self { view: collection }
    }
}

#[derive(Event)]
pub struct OnBuildView<T: Kind> {
    view: Instance<View<T>>,
}

impl<T: Kind> OnBuildView<T> {
    /// Returns the [`View`] instance associated with this event.
    pub fn view(&self) -> Instance<View<T>> {
        self.view
    }
}

fn trigger_build_view<T: Kind>(
    query: Query<Instance<T>, Without<Viewable<T>>>,
    mut commands: Commands,
) {
    for new_viewable in query.iter() {
        let view = commands
            .spawn_instance(View {
                viewable: new_viewable,
            })
            .instance();
        commands
            .entity(new_viewable.entity())
            .trigger(OnBuildView { view });
    }
}

#[deprecated]
pub fn rebuild<T: Kind>(viewable: InstanceRef<Viewable<T>>, commands: &mut Commands) {
    commands.entity(viewable.entity()).remove::<Viewable<T>>();
}
