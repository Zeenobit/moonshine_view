#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![allow(deprecated)] // TODO: Remove deprecated code

use std::fmt::{self, Debug};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::relationship::Relationship;
use moonshine_kind::{prelude::*, InstanceVec};
use moonshine_save::load::Unload;

/// Common elements for the view system.
pub mod prelude {
    pub use super::{ViewOf, Viewable};

    pub use super::{RegisterViewable, View};
}

/// Collection of utilities to extend the Moonshine Model/View framework.
#[cfg(feature = "ext")]
pub mod ext;

/// Trait used to register a [`ViewableKind`] with an [`App`].
#[deprecated]
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
#[deprecated]
pub trait ViewableKind: Kind {
    /// Returns the default view [`Bundle`] for this [`Kind`].
    ///
    /// # Usage
    /// By default, this returns an [`Unload`] component to ensure all views are despawned when the game is loaded.
    ///
    /// The output bundle is inserted into the [`View`] entity when it is spawned before [`Viewable<T>`] is inserted.
    /// This is useful for initializing the view entity before anything else can react to it.
    fn view_bundle() -> impl Bundle {
        Unload
    }
}

/// A [`Component`] which represents a view of an [`Entity`] of the given [`ViewableKind`].
///
/// A "view entity" is analogous to the View in the Model-View-Controller (MVC) pattern.
#[derive(Component)]
#[require(Unload)]
#[component(on_insert = <Self as Relationship>::on_insert)]
#[component(on_discard = <Self as Relationship>::on_discard)]
pub struct ViewOf<T: Kind>(pub Instance<T>);

#[deprecated]
pub type View<T> = ViewOf<T>;

impl<T: Kind> ViewOf<T> {
    /// Returns the associated viewable entity.
    pub fn viewable(&self) -> Instance<T> {
        self.0
    }
}

impl<T: Kind> Debug for ViewOf<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ViewOf").field(&self.0).finish()
    }
}

impl<T: Kind> Relationship for ViewOf<T> {
    type RelationshipTarget = Viewable<T>;

    fn get(&self) -> Entity {
        self.0.entity()
    }

    fn from(entity: Entity) -> Self {
        Self(unsafe { Instance::from_entity_unchecked(entity) })
    }

    fn set_risky(&mut self, entity: Entity) {
        unsafe {
            *self.0.as_entity_mut() = entity;
        }
    }
}

/// A [`Component`] which represents an [`Entity`] associated with a [`View`].
///
/// A "viewable entity" is analogous to the Model in the Model-View-Controller (MVC) pattern.
#[derive(Component)]
#[component(on_discard = <Self as RelationshipTarget>::on_discard)]
#[component(on_despawn = <Self as RelationshipTarget>::on_despawn)]
pub struct Viewable<T: Kind> {
    views: InstanceVec<ViewOf<T>>,
}

impl<T: Kind> Viewable<T> {
    /// Spawns a [`View<T>`] for a given viewable instance and returns it.
    ///
    /// This function is useful for spawning views manually when you to modify the view before the observers start reacting.
    #[deprecated]
    pub fn spawn_view<'a>(
        target: Instance<T>,
        commands: &'a mut Commands,
    ) -> InstanceCommands<'a, ViewOf<T>>
    where
        T: ViewableKind,
    {
        let view_entity = commands.spawn((T::view_bundle(), ViewOf(target)));
        unsafe { InstanceCommands::from_entity_unchecked(view_entity) }
    }

    /// Returns the [`View`] [`Instance`] associated with this [`Viewable`].
    #[deprecated]
    pub fn view(&self) -> Instance<ViewOf<T>> {
        self.views[0]
    }

    /// Returns all [`View`] instances associated with this [`Viewable`].
    pub fn views(&self) -> impl ExactSizeIterator<Item = Instance<ViewOf<T>>> + '_ {
        self.views.iter().copied()
    }
}

impl<T: Kind> RelationshipTarget for Viewable<T> {
    const LINKED_SPAWN: bool = true;

    type Relationship = ViewOf<T>;

    type Collection = InstanceVec<ViewOf<T>>;

    fn collection(&self) -> &Self::Collection {
        &self.views
    }

    fn collection_mut_risky(&mut self) -> &mut Self::Collection {
        &mut self.views
    }

    fn from_collection_risky(collection: Self::Collection) -> Self {
        Self { views: collection }
    }
}

#[deprecated]
fn trigger_build_view<T: ViewableKind>(
    query: Query<Instance<T>, Without<Viewable<T>>>,
    mut commands: Commands,
) {
    for viewable in query.iter() {
        Viewable::spawn_view(viewable, &mut commands);
    }
}
