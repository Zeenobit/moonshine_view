#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::relationship::Relationship;
use moonshine_kind::prelude::*;
use moonshine_save::load::Unload;

/// Common elements for the view system.
pub mod prelude {
    pub use super::{OnBuildView, RegisterViewable, View, Viewable, ViewableKind};
}

#[cfg(test)]
mod tests;

/// Trait used to register a [`ViewableKind`] with an [`App`].
pub trait RegisterViewable {
    /// Adds a given [`Kind`] as viewable.
    fn register_viewable<T: ViewableKind>(&mut self) -> &mut Self;
}

impl RegisterViewable for App {
    fn register_viewable<T: ViewableKind>(&mut self) -> &mut Self {
        self.add_systems(PreUpdate, trigger_build_view::<T>);
        self
    }
}

/// A trait used to define a [`Kind`] as viewable.
pub trait ViewableKind: Kind {
    /// Returns the default view [`Bundle`] for this [`Kind`].
    ///
    /// # Usage
    /// By default, this returns an [`Unload`] component to ensure all views are despawned when the game is loaded.
    ///
    /// The output bundle is inserted into the [`View`] entity when it is spawned before [`OnBuildView`] is triggered.
    /// This is useful for inserting the initial required components of the view before [`OnBuildView`].
    fn view_bundle() -> impl Bundle {
        Unload
    }
}

/// A [`Component`] which represents a view of an [`Entity`] of the given [`ViewableKind`].
///
/// A "view entity" is analogous to the View in the Model-View-Controller (MVC) pattern.
#[derive(Component)]
#[component(on_insert = <Self as Relationship>::on_insert)]
#[component(on_replace = <Self as Relationship>::on_replace)]
pub struct View<T: ViewableKind> {
    viewable: Instance<T>,
}

impl<T: ViewableKind> View<T> {
    /// Returns the associated viewable entity.
    pub fn viewable(&self) -> Instance<T> {
        self.viewable
    }
}

impl<T: ViewableKind> Relationship for View<T> {
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

/// A [`Component`] which represents an [`Entity`] associated with a [`View`].
///
/// A "viewable entity" is analogous to the Model in the Model-View-Controller (MVC) pattern.
#[derive(Component, Debug)]
#[component(on_replace = <Self as RelationshipTarget>::on_replace)]
#[component(on_despawn = <Self as RelationshipTarget>::on_despawn)]
pub struct Viewable<T: ViewableKind> {
    view: Instance<View<T>>,
}

impl<T: ViewableKind> Viewable<T> {
    /// Returns the [`View`] [`Instance`] associated with this [`Viewable`].
    pub fn view(&self) -> Instance<View<T>> {
        self.view
    }
}

impl<T: ViewableKind> RelationshipTarget for Viewable<T> {
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

/// An [`Event`] triggered when a new [`View`] is spawned for a [`Viewable`].
///
/// This event targets the [`Viewable`] [`Entity`] and provides access to the new [`View`].
#[derive(Event)]
pub struct OnBuildView<T: ViewableKind> {
    view: Instance<View<T>>,
}

impl<T: ViewableKind> OnBuildView<T> {
    /// Returns the [`View`] instance associated with this event.
    pub fn view(&self) -> Instance<View<T>> {
        self.view
    }
}

fn trigger_build_view<T: ViewableKind>(
    query: Query<Instance<T>, Without<Viewable<T>>>,
    mut commands: Commands,
) {
    for new_viewable in query.iter() {
        let view = commands
            .spawn_instance(View {
                viewable: new_viewable,
            })
            .insert(T::view_bundle())
            .instance();
        commands
            .entity(new_viewable.entity())
            .trigger(OnBuildView { view });
    }
}
