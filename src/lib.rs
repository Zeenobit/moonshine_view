#[doc = include_str!("../README.md")]
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_hierarchy::prelude::*;
use bevy_utils::{tracing::debug, HashMap, HashSet};

use moonshine_kind::prelude::*;
use moonshine_object::prelude::*;
use moonshine_save::prelude::*;

pub mod prelude {
    pub use super::{Observables, Observe, Observer, RegisterObservable, View, ViewBuilder};

    pub use moonshine_object::Object;
}

/// An extension trait used to register an observable type with an [`App`].
pub trait RegisterObservable {
    /// Registers the given type as an observable.
    fn register_observable<T: Observe>(self) -> Self;
}

impl RegisterObservable for &mut App {
    fn register_observable<T: Observe>(self) -> Self {
        self.world.init_resource::<Observables>();
        self.add_systems(PreUpdate, observe::<T>.after(LoadSystem::Load))
            .add_systems(Last, despawn::<T>)
    }
}

/// Any [`Kind`] which should be associated with a [`View`].
pub trait Observe: Kind {
    /// This function is invoked whenever a new instance of this [`Kind`] is spawned without a [`View`].
    fn observe(_world: &World, _object: Object<Self>, view: &mut ViewBuilder<Self>);
}

/// A handle to a [`View`] at the time of its creation.
pub struct ViewBuilder<'a, T: Kind>(InstanceCommands<'a, View<T>>);

impl<'a, T: Kind> ViewBuilder<'a, T> {
    /// Returns the view [`Instance`].
    pub fn instance(&self) -> Instance<View<T>> {
        self.0.instance()
    }

    /// Returns the view [`Entity`].
    pub fn entity(&self) -> Entity {
        self.0.entity()
    }

    /// Inserts a new [`Bundle`] into the [`View`] entity.
    pub fn insert(&mut self, bundle: impl Bundle) -> &mut Self {
        self.0.insert(bundle);
        self
    }

    /// Adds some children to the [`View`] entity.
    pub fn insert_children<F: FnOnce(&mut ChildBuilder)>(&mut self, f: F) -> &mut Self {
        self.0.with_children(|view| f(view));
        self
    }

    /// Adds some children to the [`View`] entity.
    #[deprecated(note = "use `insert_children` instead")]
    pub fn spawn<F: FnOnce(&mut ChildBuilder)>(&mut self, f: F) -> &mut Self {
        self.insert_children(f)
    }
}

/// A [`Component`] which associates its [`Entity`] with a [`View`] of given [`Kind`].
#[derive(Component)]
pub struct Observer<T: Kind> {
    view: Instance<View<T>>,
}

impl<T: Kind> Observer<T> {
    fn new(view: Instance<View<T>>) -> Self {
        Self { view }
    }
}

impl<T: Observe> Observer<T> {
    pub fn view(&self) -> Instance<View<T>> {
        self.view
    }
}

/// A [`Component`] which associates its [`Entity`] with an observed instance of given [`Kind`].
#[derive(Component)]
pub struct View<T: Kind> {
    target: Instance<T>,
}

impl<T: Observe> View<T> {
    pub fn target(&self) -> Instance<T> {
        self.target
    }
}

#[derive(Bundle)]
struct ViewBundle<T: Kind> {
    view: View<T>,
    unload: Unload,
}

impl<T: Kind> ViewBundle<T> {
    pub fn new(observable: impl Into<Instance<T>>) -> Self {
        let target = observable.into();
        Self {
            view: View { target },
            unload: Unload,
        }
    }
}

impl<T: Observe> KindBundle for ViewBundle<T> {
    type Kind = View<T>;
}

/// A [`Resource`] which contains a mapping of all observable entities to their observed views.
///
/// # Usage
///
/// Typically, you want to access views or observables using [`Observer`] and [`View`] components.
/// However, in some cases it may be needed to access **all** views for a given observables.
/// This [`Resource`] provides an interface for this specific purpose.
#[derive(Resource, Default)]
pub struct Observables(HashMap<Entity, HashSet<Entity>>);

impl Observables {
    /// Iterates over all observable entities with at least one observed view.
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.0.keys().copied()
    }

    /// Iterates over all observed views for a given entity.
    pub fn views(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
        self.0
            .get(&entity)
            .into_iter()
            .flat_map(|views| views.iter().copied())
    }

    fn add<T: Observe>(&mut self, entity: Entity, view: Instance<View<T>>) {
        self.0.entry(entity).or_default().insert(view.entity());
    }

    fn remove<T: Observe>(&mut self, entity: Entity, view: Instance<View<T>>) {
        let views = self.0.get_mut(&entity).unwrap();
        views.remove(&view.entity());
        if views.is_empty() {
            self.0.remove(&entity);
        }
    }
}

fn observe<T: Observe>(
    objects: Objects<T, Without<Observer<T>>>,
    world: &World,
    mut commands: Commands,
) {
    for observable in objects.iter() {
        let view = commands.spawn_instance(ViewBundle::new(observable));
        let mut view = ViewBuilder(view);
        T::observe(world, observable, &mut view);
        let view = view.instance();
        let entity = observable.entity();
        commands.add(move |world: &mut World| {
            world.resource_mut::<Observables>().add(entity, view);
        });
        commands.entity(entity).insert(Observer::new(view));
        debug!("{view:?} spawned for {entity:?}");
    }
}

fn despawn<T: Observe>(
    views: Query<InstanceRef<View<T>>>,
    mut observables: ResMut<Observables>,
    query: Query<(), T::Filter>,
    mut commands: Commands,
) {
    for (view, observable) in views.iter().map(|view| (view.instance(), view.target())) {
        if query.get(observable.entity()).is_err() {
            commands.entity(view.entity()).despawn_recursive();
            observables.remove(observable.entity(), view);
            debug!("{view:?} despawned for {observable:?}");
        }
    }
}

/// Despawns the current [`View`] associated with this [`Observer`] and rebuilds a new one.
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
/// impl Observe for Shape {
///     fn observe(world: &World, object: Object<Self>, view: &mut ViewBuilder<Self>) {
///         let shape = world.get::<Shape>(object.entity());
///         // ...
///     }
/// }
///
/// fn rebuild_shape_views(query: Query<InstanceRef<Observer<Shape>>>, mut commands: Commands) {
///     for observer in query.iter() {
///         moonshine_view::rebuild(observer, &mut commands);
///     }
/// }
/// ```
pub fn rebuild<T: Observe>(observer: InstanceRef<Observer<T>>, commands: &mut Commands) {
    let entity = observer.entity();
    let view = observer.view();
    commands.entity(view.entity()).despawn_recursive();
    commands.add(move |world: &mut World| {
        world.resource_mut::<Observables>().remove(entity, view);
    });
    commands.entity(entity).remove::<Observer<T>>();
}
