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
    /// Adds a view for a given [`Kind`].
    fn add_view<T: Kind, V: BuildView<T>>(&mut self) -> &mut Self;

    /// Adds a given [`Kind`] as viewable.
    fn add_viewable<T: BuildView>(&mut self) -> &mut Self;
}

impl RegisterView for App {
    fn add_view<T: Kind, V: BuildView<T>>(&mut self) -> &mut Self {
        self.add_systems(PreUpdate, build_view::<T, V>.after(spawn_view::<T>));
        self
    }

    /// Adds a given [`Kind`] as viewable.
    fn add_viewable<T: BuildView>(&mut self) -> &mut Self {
        let mut viewables = self
            .world_mut()
            .get_resource_or_insert_with(Viewables::default);
        #[allow(deprecated)] // TODO: Remove
        viewables.add_kind::<T>();
        self.add_systems(
            PreUpdate,
            (spawn_view::<T>.after(CheckSystems), build_view::<T, T>).chain(),
        );
        self.add_systems(Last, despawn_view::<T>);
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
    views: HashMap<Entity, Entity>,
}

impl Viewables {
    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains_key(&entity)
    }

    /// Iterates over all viewed [`Viewable`] entities.
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.keys().copied()
    }

    #[deprecated]
    pub fn is_viewable_kind<T: Kind>(&self) -> bool {
        self.kinds.contains_key(&TypeId::of::<T>())
    }

    pub fn is_view(&self, entity: Entity) -> bool {
        self.views.contains_key(&entity)
    }

    #[deprecated]
    pub fn is_view_of_kind<T: Kind>(&self, entity: Entity) -> bool {
        let Some(viewable) = self.views.get(&entity) else {
            return false;
        };
        self.kinds
            .get(&std::any::TypeId::of::<T>())
            .is_some_and(|entities| entities.contains(viewable))
    }

    /// Iterates over all views for a given [`Viewable`] [`Entity`].
    pub fn views(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
        self.entities
            .get(&entity)
            .into_iter()
            .flat_map(|views| views.iter().copied())
    }

    #[deprecated]
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
        let previous = self.views.insert(view.entity(), entity);
        debug_assert!(previous.is_none());
    }

    fn remove<T: Kind>(&mut self, entity: Entity, view: Instance<View<T>>) {
        let views = self.entities.get_mut(&entity).unwrap();
        views.remove(&view.entity());
        if views.is_empty() {
            self.entities.remove(&entity);
        }
        let kinds = self.kinds.get_mut(&TypeId::of::<T>()).unwrap();
        kinds.remove(&view.entity());
        self.views.remove(&view.entity());
    }
}

fn spawn_view<T: Kind>(objects: Objects<T, Without<Viewable<T>>>, mut commands: Commands) {
    for object in objects.iter() {
        let view_entity = commands.spawn(ViewBundle::new(object)).id();
        // SAFE: `ViewBundle` will be inserted.
        let view: Instance<View<T>> = unsafe { Instance::from_entity_unchecked(view_entity) };
        let entity = object.entity();
        commands.queue(move |world: &mut World| {
            world.resource_mut::<Viewables>().add(entity, view);
        });
        commands.entity(entity).insert(Viewable::new(view));
        debug!("{view:?} spawned for {entity:?}");
    }
}

fn build_view<T: Kind, S: BuildView<T>>(
    objects: Objects<
        T,
        (
            Or<(
                Added<Viewable<T>>,
                (With<Viewable<T>>, Without<Viewable<S>>),
            )>,
            S::Filter,
        ),
    >,
    world: &World,
    mut commands: Commands,
) {
    for object in objects.iter() {
        let base_viewable = world.get::<Viewable<T>>(object.entity()).unwrap();
        let base_view = base_viewable.view();

        // SAFE: `View<S>` will be inserted later.
        let view = unsafe { base_view.cast_into_unchecked::<View<S>>() };

        commands
            .entity(object.entity())
            .insert_if_new(Viewable::<S>::new(view));

        commands
            .entity(base_view.entity())
            .insert_if_new(View::<S> {
                // SAFE: `S::Filter` applied to query
                viewable: unsafe { object.instance().cast_into_unchecked() },
            });

        S::build(world, object, commands.instance(base_view));
    }
}

fn despawn_view<T: Kind>(
    views: Query<InstanceRef<View<T>>>,
    query: Query<(), T::Filter>,
    mut commands: Commands,
) {
    for view in views.iter() {
        let viewable = view.viewable();
        let view = view.instance();
        if query.get(viewable.entity()).is_err() {
            commands.queue(move |world: &mut World| {
                if let Ok(mut entity) = world.get_entity_mut(viewable.entity()) {
                    entity.remove::<Viewable<T>>();
                }
                if let Ok(view_entity) = world.get_entity_mut(view.entity()) {
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
///     fn build(world: &World, object: Object<Self>, view: ViewCommands<Self>) {
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
    commands.queue(move |world: &mut World| {
        world.resource_mut::<Viewables>().remove(entity, view);
    });
    commands.entity(entity).remove::<Viewable<T>>();
}
