use std::fmt;

/// A parsed url: the part routes match, plus the query and fragment.
///
/// `pathname` always starts with `/`; `search` includes its leading `?` and
/// `hash` its leading `#`, both empty when absent. Routes only ever look at
/// `pathname`, so a query string or a fragment never changes which route
/// matches.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Location {
    /// The path, always beginning with `/`.
    pub pathname: String,
    /// The query string including the leading `?`, or empty.
    pub search: String,
    /// The fragment including the leading `#`, or empty.
    pub hash: String,
}

impl Location {
    /// Parses a path or a full url.
    ///
    /// A full url is reduced to its path: the origin is dropped because a
    /// router lives inside one origin. A bare path is normalized to begin with
    /// `/`.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self::parse(&path.into())
    }

    /// Parses a path or a full url into its three parts.
    #[must_use]
    pub fn parse(url: &str) -> Self {
        let (url, hash) = split_suffix(url, '#');
        let (path, search) = split_suffix(url, '?');
        let path = strip_origin(path);
        let pathname = if path.is_empty() {
            "/".to_string()
        } else if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        Self {
            pathname,
            search,
            hash,
        }
    }

    /// Resolves `to` against this location, the way the browser would.
    ///
    /// `#fragment` keeps the current path and query, `?query` keeps the current
    /// path, an absolute path replaces everything, and anything else is
    /// appended to the current directory. Dot segments are not collapsed;
    /// the browser's own history does that on the web.
    #[must_use]
    pub fn resolve(&self, to: &str) -> Self {
        if to.is_empty() {
            return self.clone();
        }
        if let Some(fragment) = to.strip_prefix('#') {
            let mut location = self.clone();
            location.hash = format!("#{fragment}");
            return location;
        }
        if let Some(query) = to.strip_prefix('?') {
            let mut location = self.clone();
            location.search = format!("?{query}");
            location.hash.clear();
            return location;
        }
        if to.starts_with('/') || to.contains("://") {
            return Self::parse(to);
        }
        let directory = if self.pathname.ends_with('/') {
            self.pathname.as_str()
        } else {
            self.pathname
                .rsplit_once('/')
                .map_or(
                    "/",
                    |(directory, _)| {
                        if directory.is_empty() { "/" } else { directory }
                    },
                )
        };
        let base = if directory.ends_with('/') {
            directory.to_string()
        } else {
            format!("{directory}/")
        };
        Self::parse(&format!("{base}{to}"))
    }

    /// The path including the query and fragment, ready for `push`.
    #[must_use]
    pub fn path(&self) -> String {
        format!("{}{}{}", self.pathname, self.search, self.hash)
    }

    /// The query string without its leading `?`.
    #[must_use]
    pub fn query(&self) -> &str {
        self.search.strip_prefix('?').unwrap_or(&self.search)
    }

    /// The fragment without its leading `#`.
    #[must_use]
    pub fn fragment(&self) -> &str {
        self.hash.strip_prefix('#').unwrap_or(&self.hash)
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.path())
    }
}

impl From<&str> for Location {
    fn from(value: &str) -> Self {
        Self::parse(value)
    }
}

impl From<String> for Location {
    fn from(value: String) -> Self {
        Self::parse(&value)
    }
}

fn split_suffix(value: &str, separator: char) -> (&str, String) {
    match value.split_once(separator) {
        Some((before, after)) => (before, format!("{separator}{after}")),
        None => (value, String::new()),
    }
}

fn strip_origin(path: &str) -> &str {
    let Some((_, rest)) = path.split_once("://") else {
        return path;
    };
    match rest.find('/') {
        Some(index) => &rest[index..],
        // `https://example.com` has the path `/`, not an empty one.
        None => "/",
    }
}

#[cfg(test)]
mod tests {
    use super::Location;

    #[test]
    fn parses_path_query_and_fragment() {
        let location = Location::parse("/users/42?tab=posts#top");
        assert_eq!(location.pathname, "/users/42");
        assert_eq!(location.search, "?tab=posts");
        assert_eq!(location.hash, "#top");
        assert_eq!(location.query(), "tab=posts");
        assert_eq!(location.fragment(), "top");
    }

    #[test]
    fn a_full_url_is_reduced_to_its_path() {
        let location = Location::parse("https://example.com/users/42?tab=posts");
        assert_eq!(location.pathname, "/users/42");
        assert_eq!(location.search, "?tab=posts");

        let origin_only = Location::parse("https://example.com");
        assert_eq!(origin_only.pathname, "/");
    }

    #[test]
    fn a_bare_path_gets_a_leading_slash() {
        assert_eq!(Location::parse("users/42").pathname, "/users/42");
    }

    #[test]
    fn a_fragment_only_target_keeps_the_path_and_query() {
        let current = Location::parse("/users/42?tab=posts");
        let target = current.resolve("#top");
        assert_eq!(target.pathname, "/users/42");
        assert_eq!(target.search, "?tab=posts");
        assert_eq!(target.hash, "#top");
    }

    #[test]
    fn a_query_only_target_keeps_the_path() {
        let current = Location::parse("/users/42?tab=posts#top");
        let target = current.resolve("?tab=followers");
        assert_eq!(target.pathname, "/users/42");
        assert_eq!(target.search, "?tab=followers");
        assert_eq!(target.hash, "", "a new query drops the old fragment");
    }

    #[test]
    fn a_relative_target_is_appended_to_the_current_directory() {
        let current = Location::parse("/users/42");
        assert_eq!(current.resolve("posts").pathname, "/users/posts");
        assert_eq!(current.resolve("posts").path(), "/users/posts");
    }

    #[test]
    fn a_relative_target_replaces_the_last_segment_like_the_browser() {
        assert_eq!(
            Location::parse("/users").resolve("posts").pathname,
            "/posts",
            "a non-directory path loses its last segment"
        );
        assert_eq!(
            Location::parse("/users/").resolve("posts").pathname,
            "/users/posts",
            "a trailing slash keeps the directory"
        );
        assert_eq!(Location::parse("/").resolve("posts").pathname, "/posts");
    }

    #[test]
    fn an_absolute_target_replaces_the_current_one() {
        let current = Location::parse("/users/42?tab=posts");
        let target = current.resolve("/about?x=1");
        assert_eq!(target.pathname, "/about");
        assert_eq!(target.search, "?x=1");
        assert_eq!(target.hash, "");
    }

    #[test]
    fn path_round_trips() {
        let location = Location::parse("/users/42?tab=posts#top");
        assert_eq!(location.path(), "/users/42?tab=posts#top");
        assert_eq!(Location::parse(&location.path()), location);
    }
}
