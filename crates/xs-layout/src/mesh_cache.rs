//! Reusing a mesh between the frames that want the same one.

use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::vector::VectorGeometry;

/// How many meshes to keep.
///
/// A screen is a handful of nodes - the vector example is nine cards and a tiger
/// slide - and a node is one entry however many frames it stays up. Past this
/// count the entries belong to nodes that have already left the tree, so their
/// meshes are not coming back and holding them costs memory for nothing.
///
/// Small on purpose. The lookup is a linear scan, and at a hundred entries the
/// scan would start to matter next to the work it exists to avoid.
pub const CACHE_CAPACITY: usize = 16;

/// Compares two opaque keys, or says they cannot be compared at all.
///
/// A function pointer rather than a closure so that `Entry` stays a plain struct
/// the cache can hold without naming a key type - see [`MeshCache::get_or_build`],
/// where the key type is only known to be `PartialEq` at the call site.
type KeyEq = fn(&dyn Any, &dyn Any) -> bool;

/// One built mesh and the key it was built from.
///
/// The key is erased, which is the price of a cache that holds a key type from a
/// crate above this one without that crate being able to name this cache's
/// generic parameter. Nothing is lost but the ability to name the type, and the
/// comparator travels with the key.
struct Entry {
    key: Box<dyn Any>,
    eq: KeyEq,
    geometry: Rc<VectorGeometry>,
}

/// A small cache of built meshes, keyed on an exact comparison.
///
/// # Why the key is compared and not hashed
///
/// A hash is the obvious key, and for this it is the wrong one. A collision means
/// silently drawing the wrong artwork, and there is no way to notice and nothing
/// to report. Storing the key and comparing it costs one pass over the input -
/// microseconds, against the milliseconds of flattening and triangulation it
/// replaces - and makes a hit exact rather than almost certain.
///
/// # Why it is not thread safe, and does not need to be
///
/// A mesh is built during layout, on the thread that is laying out, and the
/// element tree it comes from holds a `Box<dyn Animate>` and so is not `Send`
/// either. Interior mutability with no lock is the whole design; a shared cache
/// would mean a lock on the hottest path in the layout pass.
#[derive(Default)]
pub struct MeshCache {
    /// Newest last, so eviction is a `remove(0)` and a hit is a `remove` off the
    /// back.
    entries: RefCell<Vec<Entry>>,
}

impl MeshCache {
    /// An empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The mesh for `key`, built by `build` only if it is not built already.
    ///
    /// The [`Rc`] is shared rather than copied, so a caller that keeps the mesh -
    /// `Material::Vector` does - shares the vertex buffers instead of duplicating
    /// them.
    ///
    /// `key` is compared with [`PartialEq`], and the comparison happens on a hit
    /// and on a miss alike: a key that is equal to none of them still gets built,
    /// and one that equals an entry never is.
    pub fn get_or_build<K: PartialEq + 'static>(
        &self,
        key: K,
        build: impl FnOnce() -> Rc<VectorGeometry>,
    ) -> Rc<VectorGeometry> {
        let eq: KeyEq = |a, b| match (a.downcast_ref::<K>(), b.downcast_ref::<K>()) {
            (Some(a), Some(b)) => a == b,
            // Two keys of some *other* type are not equal, even though both
            // downcasts failed and would otherwise compare as `None == None`.
            _ => false,
        };

        let mut entries = self.entries.borrow_mut();
        if let Some(index) = entries
            .iter()
            .rposition(|entry| (entry.eq)(entry.key.as_ref(), &key))
        {
            let entry = entries.remove(index);
            let geometry = Rc::clone(&entry.geometry);
            entries.push(entry);
            return geometry;
        }

        let geometry = build();
        entries.push(Entry {
            key: Box::new(key),
            eq,
            geometry: Rc::clone(&geometry),
        });
        if entries.len() > CACHE_CAPACITY {
            entries.remove(0);
        }
        geometry
    }

    /// How many meshes are held.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.borrow().len()
    }

    /// Whether the cache holds nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.borrow().is_empty()
    }

    /// Forgets everything in it.
    ///
    /// For tests, which are otherwise at the mercy of each other's nodes: two
    /// tests that build the same circle at the same size would share an entry, and
    /// one asserting that a second build happened would pass for the wrong reason.
    pub fn clear(&self) {
        self.entries.borrow_mut().clear();
    }
}

/// Written by hand because the keys are erased and have nothing to print.
impl fmt::Debug for MeshCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MeshCache").field("len", &self.len()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rectangle, Vector2};

    fn rect() -> Rc<VectorGeometry> {
        Rc::new(VectorGeometry {
            bounds: Rectangle::new(0., 0., 10., 10.),
            ..Default::default()
        })
    }

    #[derive(PartialEq, Eq, Debug, Clone)]
    struct Key {
        size: Vector2,
        fill: u8,
    }

    #[test]
    fn the_same_key_gives_the_same_mesh_back() {
        let cache = MeshCache::new();
        let key = Key { size: Vector2::new(10., 10.), fill: 1 };
        let first = cache.get_or_build(key.clone(), rect);
        let second = cache.get_or_build(key, rect);
        assert!(Rc::ptr_eq(&first, &second), "the second call rebuilt");
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn a_key_one_field_different_does_not_hit() {
        let cache = MeshCache::new();
        let first = cache.get_or_build(Key { size: Vector2::new(10., 10.), fill: 1 }, rect);
        let bigger = cache.get_or_build(Key { size: Vector2::new(20., 10.), fill: 1 }, rect);
        let other = cache.get_or_build(Key { size: Vector2::new(10., 10.), fill: 2 }, rect);
        assert!(!Rc::ptr_eq(&first, &bigger));
        assert!(!Rc::ptr_eq(&first, &other));
    }

    /// Two keys of different types cannot be compared, and must not be reported
    /// as equal just because both of them failed to downcast.
    #[test]
    fn keys_of_different_types_are_never_equal() {
        let cache = MeshCache::new();
        cache.get_or_build(Key { size: Vector2::new(1., 1.), fill: 0 }, rect);
        cache.get_or_build(7u32, rect);
        assert_eq!(cache.len(), 2, "the two keys collided");
    }

    /// The point of the eviction: a node whose entry has been pushed out must not
    /// be handed its old mesh, or a shape that changed would keep drawing what was
    /// on screen last frame.
    #[test]
    fn an_evicted_mesh_is_not_handed_out_again() {
        let cache = MeshCache::new();
        let key = Key { size: Vector2::new(1., 1.), fill: 0 };
        let first = cache.get_or_build(key.clone(), rect);
        for size in 1..=CACHE_CAPACITY + 2 {
            cache.get_or_build(
                Key { size: Vector2::new(size as f32, 1.), fill: 9 },
                rect,
            );
        }
        let again = cache.get_or_build(key, rect);
        assert!(!Rc::ptr_eq(&first, &again), "an evicted mesh came back");
        assert!(cache.len() <= CACHE_CAPACITY);
    }

    #[test]
    fn clearing_empties_it() {
        let cache = MeshCache::new();
        assert!(cache.is_empty());
        cache.get_or_build(Key { size: Vector2::new(1., 1.), fill: 0 }, rect);
        assert_eq!(cache.len(), 1);
        cache.clear();
        assert!(cache.is_empty());
    }
}