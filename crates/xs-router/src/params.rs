use std::{collections::BTreeMap, ops::Index};

/// The dynamic segments matched for a route.
///
/// A route pattern `:id` stores under `id`, and a `*` splat stores the whole
/// remainder under both `*` and `splat`. Parameters inherited from a parent
/// layout are present in a child's params too, and a child wins if both declare
/// the same name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Params {
    values: BTreeMap<String, String>,
}

impl Params {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a parameter, returning the value it replaced.
    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<String>) -> Option<String> {
        self.values.insert(name.into(), value.into())
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(String::as_str)
    }

    /// The parameter, or `""` when it is absent.
    ///
    /// This is what keeps a component from panicking on a route it is also
    /// mounted under without the parameter, which is the common shape for a
    /// layout that renders the same piece for an index route.
    #[must_use]
    pub fn param(&self, name: &str) -> &str {
        self.get(name).unwrap_or("")
    }

    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.values
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }

    pub(crate) fn merge(&mut self, other: &Self) {
        for (name, value) in &other.values {
            self.values.insert(name.clone(), value.clone());
        }
    }
}

impl Index<&str> for Params {
    type Output = str;

    fn index(&self, name: &str) -> &Self::Output {
        self.param(name)
    }
}

impl<'a> IntoIterator for &'a Params {
    type Item = (&'a str, &'a str);
    type IntoIter = std::iter::Map<
        std::collections::btree_map::Iter<'a, String, String>,
        fn((&'a String, &'a String)) -> (&'a str, &'a str),
    >;

    fn into_iter(self) -> Self::IntoIter {
        self.values
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }
}

impl FromIterator<(String, String)> for Params {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        Self {
            values: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Params;

    #[test]
    fn a_missing_parameter_reads_as_an_empty_string() {
        let params = Params::new();
        assert_eq!(params.get("id"), None);
        assert_eq!(params.param("id"), "");
        assert_eq!(&params["id"], "");
    }

    #[test]
    fn merged_params_keep_the_child_value_on_a_collision() {
        let mut parent = Params::new();
        parent.insert("id", "parent");
        parent.insert("section", "users");

        let mut child = Params::new();
        child.insert("id", "child");

        parent.merge(&child);
        assert_eq!(parent.param("id"), "child");
        assert_eq!(parent.param("section"), "users");
    }
}
