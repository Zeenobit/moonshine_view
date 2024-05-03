#[doc = include_str!("../README.md")]
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_hierarchy::prelude::*;
use bevy_utils::{tracing::debug, HashMap, HashSet};

use moonshine_kind::prelude::*;
use moonshine_object::prelude::*;
use moonshine_save::prelude::*;

pub mod prelude {
    pub use super::{BuildView, Model, RegisterView, View, ViewBuilder, Viewables};

    pub use moonshine_object::Object;

    #[deprecated(note = "use `RegisterViewer` instead")]
    pub use super::RegisterView as RegisterObservable;

    #[deprecated(note = "use `BuildView` instead")]
    pub use super::BuildView as Observe;

    #[deprecated(note = "use `Model` instead")]
    pub use super::Model as Observer;

    #[deprecated(note = "use `Viewables` instead")]
    pub use super::Viewables as Observables;
}

/// Extension trait used to register views using an [`App`].
pub trait RegisterView: Sized {
    /// Registers a view for a given [`Kind`].
    fn register_view<T: Kind, V: BuildView<T>>(self) -> Self;

    /// Registers a given [`Kind`] as viewable.
    fn register_viewable<T: BuildView>(self) -> Self {
        self.register_view::<T, T>()
    }

    #[deprecated(note = "use `register_viewable` instead")]
    fn register_observable<T: BuildView>(self) -> Self {
        self.register_viewable::<T>()
    }
}

impl RegisterView for &mut App {
    fn register_view<T: Kind, V: BuildView<T>>(self) -> Self {
        self.init_resource::<Viewables>()
            .add_systems(PreUpdate, spawn::<T, V>.after(LoadSystem::Load))
            .add_systems(Last, despawn::<T>)
    }
}

/// Trait used to spawn a [`View`] [`Entity`] for an [`Instance`] of [`Kind`] `T`.
pub trait BuildView<T: Kind = Self>: Kind {
    /// Called when a new [`Instance`] of [`Kind`] `T` is spawned without a [`View`].
    ///
    /// Remember to register this type using [`RegisterView`] for this to happen.
    fn build(_world: &World, _object: Object<T>, view: &mut ViewBuilder<T>);
}

/// Used to build a [`View`] [`Entity`] for a given [`Instance`] of [`Kind`] `T`.
///
/// See [`BuildView`] for more information.
pub struct ViewBuilder<'a, T: Kind>(InstanceCommands<'a, View<T>>);

impl<'a, T: Kind> ViewBuilder<'a, T> {
    /// Returns the [`View`] [`Instance`].
    pub fn instance(&self) -> Instance<View<T>> {
        self.0.instance()
    }

    /// Returns the [`View`] [`Entity`].
    pub fn entity(&self) -> Entity {
        self.0.entity()
    }

    /// Inserts a new [`Bundle`] into the [`View`] [`Entity`].
    pub fn insert(&mut self, bundle: impl Bundle) -> &mut Self {
        self.0.insert(bundle);
        self
    }

    /// Adds some children to the [`View`] [`Entity`].
    pub fn insert_children<F: FnOnce(&mut ChildBuilder)>(&mut self, f: F) -> &mut Self {
        self.0.with_children(|view| f(view));
        self
    }
}

/// [`Component`] of an [`Entity`] associated with a [`View`].
#[derive(Component)]
pub struct Model<T: Kind> {
    view: Instance<View<T>>,
}

impl<T: Kind> Model<T> {
    fn new(view: Instance<View<T>>) -> Self {
        Self { view }
    }
}

impl<T: Kind> Model<T> {
    /// Returns the [`View`] [`Instance`] associated with this [`Model`].
    pub fn view(&self) -> Instance<View<T>> {
        self.view
    }
}

/// [`Component`] of an [`Entity`] associated with a [`Model`].
#[derive(Component)]
pub struct View<T: Kind> {
    model: Instance<T>,
}

impl<T: Kind> View<T> {
    /// Returns the [`Model`] [`Instance`] associated with this [`View`].
    pub fn model(&self) -> Instance<T> {
        self.model
    }

    #[deprecated(note = "use `model` instead")]
    pub fn target(&self) -> Instance<T> {
        self.model()
    }
}

#[derive(Bundle)]
struct ViewBundle<T: Kind> {
    view: View<T>,
    unload: Unload,
}

impl<T: Kind> ViewBundle<T> {
    pub fn new(model: impl Into<Instance<T>>) -> Self {
        let model = model.into();
        Self {
            view: View { model },
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
/// Typically, you want to access models or views using [`Model`] and [`View`] components.
/// However, in some cases it may be needed to access **all** views for a given model.
/// This [`Resource`] provides an interface for this specific purpose.
#[derive(Resource, Default)]
pub struct Viewables(HashMap<Entity, HashSet<Entity>>);

impl Viewables {
    pub fn contains(&self, entity: Entity) -> bool {
        self.0.contains_key(&entity)
    }

    /// Iterates over all viewed [`Model`] entities.
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.0.keys().copied()
    }

    /// Iterates over all views for a given [`Model`] [`Entity`].
    pub fn views(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
        self.0
            .get(&entity)
            .into_iter()
            .flat_map(|views| views.iter().copied())
    }

    fn add<T: Kind>(&mut self, entity: Entity, view: Instance<View<T>>) {
        self.0.entry(entity).or_default().insert(view.entity());
    }

    fn remove<T: Kind>(&mut self, entity: Entity, view: Instance<View<T>>) {
        let views = self.0.get_mut(&entity).unwrap();
        views.remove(&view.entity());
        if views.is_empty() {
            self.0.remove(&entity);
        }
    }
}

fn spawn<T: Kind, S: BuildView<T>>(
    objects: Objects<T, (Without<Model<T>>, S::Filter)>,
    world: &World,
    mut commands: Commands,
) {
    for object in objects.iter() {
        let view = commands.spawn_instance(ViewBundle::new(object));
        let mut view = ViewBuilder(view);
        S::build(world, object, &mut view);
        let view = view.instance();
        let entity = object.entity();
        commands.add(move |world: &mut World| {
            world.resource_mut::<Viewables>().add(entity, view);
        });
        commands.entity(entity).insert(Model::new(view));
        debug!("{view:?} spawned for {entity:?}");
    }
}

fn despawn<T: Kind>(
    views: Query<InstanceRef<View<T>>>,
    query: Query<(), T::Filter>,
    mut commands: Commands,
) {
    for view in views.iter() {
        let model = view.model();
        let view = view.instance();
        if query.get(model.entity()).is_err() {
            if let Some(mut entity) = commands.get_entity(model.entity()) {
                entity.remove::<Model<T>>();
            }
            commands.entity(view.entity()).despawn_recursive();
            commands.add(move |world: &mut World| {
                world
                    .resource_mut::<Viewables>()
                    .remove(model.entity(), view);
            });
            debug!("{view:?} despawned for {model:?}");
        }
    }
}

/// Despawns the current [`View`] associated with this [`Model`] and rebuilds a new one.
///
/// # Example
/// ```
/// # use bevy::prelude::*;
/// # use moonshine_view::prelude::*;
/// # use moonshine_kind::prelude::*; // For `intsance_ref` method
///
/// #[derive(Component)]
/// enum Shape {
///     Square,
///     Circle,
/// }
///
/// impl BuildView for Shape {
///     fn build(world: &World, object: Object<Self>, view: &mut ViewBuilder<Self>) {
///         let shape = world.get::<Shape>(object.entity());
///         // ...
///     }
/// }
///
/// fn rebuild_shape_views(query: Query<InstanceRef<Model<Shape>>>, mut commands: Commands) {
///     for model in query.iter() {
///         moonshine_view::rebuild(model, &mut commands);
///     }
/// }
/// ```
pub fn rebuild<T: Kind>(model: InstanceRef<Model<T>>, commands: &mut Commands) {
    let entity = model.entity();
    let view = model.view();
    commands.entity(view.entity()).despawn_recursive();
    commands.add(move |world: &mut World| {
        world.resource_mut::<Viewables>().remove(entity, view);
    });
    commands.entity(entity).remove::<Model<T>>();
}
