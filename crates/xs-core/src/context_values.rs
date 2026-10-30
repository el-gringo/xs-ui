use std::{
    any::{Any, TypeId},
    rc::Rc,
};

/// Values handed down the widget tree.
///
/// This is the framework's `useContext`: a value is keyed by its type, so
/// providing a `Theme` makes it visible to every descendant through
/// [`Context::use_context`](crate::Context::use_context). Scopes nest, and the
/// innermost value of a type wins, which is what lets a nested route shadow its
/// parent's route context.
///
/// The store is persistent: [`provide`](Self::provide) returns a new scope and
/// leaves the old one untouched, so a value provided by one element cannot be
/// seen by that element's siblings. [`AppContext::contexts`](crate::AppContext)
/// holds the root scope, which is where an app puts values that every element
/// should see.
#[derive(Clone, Default)]
pub struct ContextValues {
    parent: Option<Rc<Self>>,
    value: Option<(TypeId, Rc<dyn Any>)>,
}

impl ContextValues {
    /// A new scope with `value` as the innermost value of its type.
    #[must_use]
    pub fn provide<T: 'static>(&self, value: T) -> Self {
        Self {
            parent: Some(Rc::new(self.clone())),
            value: Some((TypeId::of::<T>(), Rc::new(value))),
        }
    }

    /// The innermost value of type `T`, if any scope provided one.
    #[must_use]
    pub fn get<T: 'static>(&self) -> Option<Rc<T>> {
        if let Some((type_id, value)) = &self.value
            && *type_id == TypeId::of::<T>()
        {
            return Some(
                Rc::clone(value)
                    .downcast::<T>()
                    .expect("ContextValues stored a value under the wrong TypeId"),
            );
        }
        self.parent.as_ref()?.get::<T>()
    }

    /// The innermost value of type `T`.
    ///
    /// # Panics
    ///
    /// Panics when no ancestor provided one.
    #[must_use]
    #[track_caller]
    pub fn expect<T: 'static>(&self) -> Rc<T> {
        self.get::<T>().unwrap_or_else(|| {
            panic!(
                "no context of type `{}` was provided by an ancestor",
                std::any::type_name::<T>()
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::ContextValues;

    #[derive(Debug, PartialEq)]
    struct Theme(&'static str);

    #[test]
    fn a_provided_value_reaches_descendants_but_not_the_parent_scope() {
        let root = ContextValues::default();
        assert!(root.get::<Theme>().is_none());

        let child = root.provide(Theme("dark"));
        assert_eq!(child.expect::<Theme>().0, "dark");
        assert!(
            root.get::<Theme>().is_none(),
            "providing returns a new scope and leaves the old one alone"
        );
    }

    #[test]
    fn the_innermost_value_of_a_type_wins() {
        let root = ContextValues::default().provide(Theme("light"));
        let child = root.provide(Theme("dark"));
        let grandchild = child.provide(Theme("sepia"));

        assert_eq!(root.expect::<Theme>().0, "light");
        assert_eq!(child.expect::<Theme>().0, "dark");
        assert_eq!(grandchild.expect::<Theme>().0, "sepia");
    }

    #[test]
    fn different_types_do_not_shadow_each_other() {
        let values = ContextValues::default()
            .provide(Theme("dark"))
            .provide(7u32);

        assert_eq!(values.expect::<Theme>().0, "dark");
        assert_eq!(*values.expect::<u32>(), 7);
        let theme: Rc<Theme> = values.expect::<Theme>();
        assert_eq!(theme.0, "dark");
    }

    #[test]
    #[should_panic(expected = "no context of type")]
    fn expecting_a_missing_context_panics() {
        let _ = ContextValues::default().expect::<Theme>();
    }
}
