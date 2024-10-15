#![doc = include_str!("../README.md")]

use std::any::TypeId;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_hierarchy::prelude::*;
use bevy_utils::{tracing::debug, HashMap, HashSet};

use moonshine_core::{check::CheckSystems, prelude::*};

pub mod prelude {
    pub use super::{BuildView, RegisterView, View, ViewCommands, Viewable, Viewables};
}

/// Extension trait used to register views using an [`App`].
pub trait RegisterView {
    /// Registers a view for a given [`Kind`].
    fn register_view<T: Kind, V: BuildView<T>>(&mut self) -> &mut Self;

    /// Registers a given [`Kind`] as viewable.
    fn register_viewable<T: BuildView>(&mut self) -> &mut Self {
        self.register_view::<T, T>()
    }
}

impl RegisterView for App {
    fn register_view<T: Kind, V: BuildView<T>>(&mut self) -> &mut Self {
        self.add_systems(PreUpdate, spawn::<T, V>.after(CheckSystems));
        let mut viewables = self
            .world_mut()
            .get_resource_or_insert_with(Viewables::default);
        if !viewables.is_viewable_kind::<T>() {
            viewables.add_kind::<T>();
            self.add_systems(Last, despawn::<T>);
        }
        self
    }
}

/// Trait used to spawn a [`View`] [`Entity`] for an [`Instance`] of [`Kind`] `T`.
pub trait BuildView<T: Kind = Self>: Kind {
    /// Called when a new [`Instance`] of [`Kind`] `T` is spawned without a [`View`].
    ///
    /// Remember to register this type using [`RegisterView`] for this to happen.
    fn build(_world: &World, _object: Object<T>, view: ViewCommands<T>);
}

pub type ViewCommands<'a, T> = InstanceCommands<'a, View<T>>;

/// [`Component`] of an [`Entity`] associated with a [`View`].
#[derive(Component)]
pub struct Viewable<T: Kind> {
    view: Instance<View<T>>,
}

impl<T: Kind> Viewable<T> {
    fn new(view: Instance<View<T>>) -> Self {
        Self { view }
    }
}

#[deprecated(note = "Use `Viewable` instead")]
pub type Model<T> = Viewable<T>;

impl<T: Kind> Viewable<T> {
    /// Returns the [`View`] [`Instance`] associated with this [`Viewable`].
    pub fn view(&self) -> Instance<View<T>> {
        self.view
    }
}

/// [`Component`] of an [`Entity`] associated with a [`Viewable`].
#[derive(Component)]
pub struct View<T: Kind> {
    viewable: Instance<T>,
}

impl<T: Kind> View<T> {
    #[deprecated(note = "Use `viewable` instead")]
    pub fn model(&self) -> Instance<T> {
        self.viewable
    }

    /// Returns the associated viewable entity.
    pub fn viewable(&self) -> Instance<T> {
        self.viewable
    }
}

#[derive(Bundle)]
struct ViewBundle<T: Kind> {
    view: View<T>,
    unload: Unload,
}

impl<T: Kind> ViewBundle<T> {
    pub fn new(viewable: impl Into<Instance<T>>) -> Self {
        Self {
            view: View {
                viewable: viewable.into(),
            },
            unload: Unload,
        }
    }
}

impl<T: Kind> KindBundle for ViewBundle<T> {
    type Kind = View<T>;
}

/// A [`Resource`] which contains a mapping of all viewable entities to their views.
///
/// # Usage
///
/// Typically, you want to access viewables or views using [`Viewable`] and [`View`] components.
/// However, in some cases it may be needed to access **all** views for a given viewable.
/// This [`Resource`] provides an interface for this specific purpose.
#[derive(Resource, Default)]
pub struct Viewables {
    entities: HashMap<Entity, HashSet<Entity>>,
    kinds: HashMap<TypeId, HashSet<Entity>>,
}

impl Viewables {
    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains_key(&entity)
    }

    /// Iterates over all viewed [`Viewable`] entities.
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.keys().copied()
    }

    pub fn is_viewable_kind<T: Kind>(&self) -> bool {
        self.kinds.contains_key(&TypeId::of::<T>())
    }

    /// Iterates over all views for a given [`Viewable`] [`Entity`].
    pub fn views(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
        self.entities
            .get(&entity)
            .into_iter()
            .flat_map(|views| views.iter().copied())
    }

    fn add_kind<T: Kind>(&mut self) {
        self.kinds.insert(TypeId::of::<T>(), HashSet::default());
    }

    fn add<T: Kind>(&mut self, entity: Entity, view: Instance<View<T>>) {
        self.entities
            .entry(entity)
            .or_default()
            .insert(view.entity());
        self.kinds
            .get_mut(&TypeId::of::<T>())
            .expect("kind must be registered as viewable")
            .insert(entity);
    }

    fn remove<T: Kind>(&mut self, entity: Entity, view: Instance<View<T>>) {
        let views = self.entities.get_mut(&entity).unwrap();
        views.remove(&view.entity());
        if views.is_empty() {
            self.entities.remove(&entity);
        }
        let kinds = self.kinds.get_mut(&TypeId::of::<T>()).unwrap();
        kinds.remove(&view.entity());
    }
}

fn spawn<T: Kind, S: BuildView<T>>(
    objects: Objects<T, (Without<Viewable<T>>, S::Filter)>,
    world: &World,
    mut commands: Commands,
) {
    for object in objects.iter() {
        let mut view = commands.spawn_instance(ViewBundle::new(object));
        S::build(world, object, view.reborrow());
        let view = view.instance();
        let entity = object.entity();
        commands.add(move |world: &mut World| {
            world.resource_mut::<Viewables>().add(entity, view);
        });
        commands.entity(entity).insert(Viewable::new(view));
        debug!("{view:?} spawned for {entity:?}");
    }
}

fn despawn<T: Kind>(
    views: Query<InstanceRef<View<T>>>,
    query: Query<(), T::Filter>,
    mut commands: Commands,
) {
    for view in views.iter() {
        let viewable = view.viewable();
        let view = view.instance();
        if query.get(viewable.entity()).is_err() {
            commands.add(move |world: &mut World| {
                if let Some(mut entity) = world.get_entity_mut(viewable.entity()) {
                    entity.remove::<Viewable<T>>();
                }
                if let Some(view_entity) = world.get_entity_mut(view.entity()) {
                    view_entity.despawn_recursive();
                }
                world
                    .resource_mut::<Viewables>()
                    .remove(viewable.entity(), view);
                debug!("{view:?} despawned for {viewable:?}");
            });
        }
    }
}

/// Despawns the current [`View`] associated with this [`Viewable`] and rebuilds a new one.
///
/// # Example
/// ```
/// # use bevy::prelude::*;
/// # use moonshine_core::prelude::*;
/// # use moonshine_view::prelude::*;
///
/// #[derive(Component)]
/// enum Shape {
///     Square,
///     Circle,
/// }
///
/// impl BuildView for Shape {
///     fn build(world: &World, object: Object<Self>, view: &mut ViewCommands<Self>) {
///         let shape = world.get::<Shape>(object.entity());
///         // ...
///     }
/// }
///
/// fn rebuild_shape_views(query: Query<InstanceRef<Viewable<Shape>>>, mut commands: Commands) {
///     for viewable in query.iter() {
///         moonshine_view::rebuild(viewable, &mut commands);
///     }
/// }
/// ```
pub fn rebuild<T: Kind>(viewable: InstanceRef<Viewable<T>>, commands: &mut Commands) {
    let entity = viewable.entity();
    let view = viewable.view();
    commands.entity(view.entity()).despawn_recursive();
    commands.add(move |world: &mut World| {
        world.resource_mut::<Viewables>().remove(entity, view);
    });
    commands.entity(entity).remove::<Viewable<T>>();
}
