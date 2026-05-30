use crate::prelude::*;
use bevy_ecs::prelude::*;
use bevy_transform::prelude::*;
use moonshine_kind::prelude::*;
use moonshine_object::prelude::*;

/// A [`System`] which updates the [`Transform`] of a [`View<T>`] from its associated [`Viewable<T>`].
pub fn push_transform<T: ViewableKind>(
    viewables: Query<InstanceRef<Viewable<T>>>,
    mut transform_query: Query<&mut Transform>,
) {
    for viewable in viewables.iter() {
        let Ok(viewable_transform) = transform_query.get_mut(viewable.entity()) else {
            continue;
        };
        if !viewable_transform.is_changed() {
            continue;
        }
        let transform = *viewable_transform;
        if let Ok(mut view_transform) = transform_query.get_mut(viewable.view().entity()) {
            *view_transform = transform;
        };
    }
}

/// A [`System`] which updates a hierarchy of [`View`] entities to copy their associated [`Viewable`] hierarchy.
///
/// Given a [`Kind`] T and P (parent), any [`Viewable<T>`] with a [`Viewable<P>`] ancestor will have its [`View<T>`]
/// attached to the associated [`View<P>`].
pub fn push_hierarchy<T: ViewableKind, P: ViewableKind>(
    parents_changed: Objects<
        Viewable<T>,
        Or<(
            Changed<ChildOf>,
            Added<ChildOf>,
            (Added<Viewable<T>>, With<ChildOf>),
        )>,
    >,
    viewable: Query<&Viewable<T>>,
    mut parents_removed: RemovedComponents<ChildOf>,
    parent_viewable: Query<&Viewable<P>>,
    mut commands: Commands,
) {
    for object in parents_changed.iter() {
        let Ok(viewable) = viewable.get(object.entity()) else {
            continue;
        };
        let Some(parent_viewable) = object.query_ancestors(&parent_viewable).next() else {
            continue;
        };
        let child_view = viewable.view();
        let parent_view = parent_viewable.view();
        commands
            .entity(child_view.entity())
            .insert(ChildOf(parent_view.entity()));
    }

    for entity in parents_removed.read() {
        let Ok(viewable) = viewable.get(entity) else {
            continue;
        };
        commands
            .entity(viewable.view().entity())
            .remove_parent_in_place();
    }
}
