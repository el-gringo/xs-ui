use std::{any::TypeId, marker::PhantomData};

#[derive(Copy, Clone)]
pub struct EntityId;

/// A dynamically typed reference to a entity, which can be downcast into a `Entity<T>`.
pub struct AnyEntity {
    pub(crate) entity_id: EntityId,
    pub(crate) entity_type: TypeId
}


impl AnyEntity {
    fn new(id: EntityId, entity_type: TypeId) -> Self {
        Self {
            entity_id: id,
            entity_type
        }
    }

    /// Returns the id associated with this entity.
    #[inline]
    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    /// Returns the [TypeId] associated with this entity.
    #[inline]
    pub fn entity_type(&self) -> TypeId {
        self.entity_type
    }
}

pub struct Entity<T> {
    pub(crate) any_entity: AnyEntity,
    pub(crate) entity_type: PhantomData<fn(T) -> T>,
}

impl<T: 'static> Entity<T> {
    #[inline]
    fn new(id: EntityId) -> Self
    where
        T: 'static,
    {
        Self {
            any_entity: AnyEntity::new(id, TypeId::of::<T>()),
            entity_type: PhantomData,
        }
    }

    /// Get the entity ID associated with this entity
    #[inline]
    pub fn entity_id(&self) -> EntityId {
        self.any_entity.entity_id
    }

    /// Convert this into a dynamically typed entity.
    #[inline]
    pub fn into_any(self) -> AnyEntity {
        self.any_entity
    }

    /// Grab a reference to this entity from the context.
    #[inline]
    pub fn read<'a>(&self, cx: &'a AppContext) -> &'a T {
        cx.entities.read(self)
    }
}

impl<T> From<Entity<T>> for AnyEntity {
    #[inline]
    fn from(entity: Entity<T>) -> Self {
        entity.any_entity
    }
}
