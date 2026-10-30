use std::{cmp::Ordering, rc::Rc};

use xs_core::AnyElement;

use crate::{Params, Route, pattern::RANK_END};

/// One node of a successful match.
#[derive(Clone)]
pub(crate) struct MatchedRoute {
    pub(crate) component: AnyElement,
    /// Parameters visible at this depth, parents included.
    pub(crate) params: Params,
    /// The full pattern up to this node, e.g. `/users/:id`.
    pub(crate) path: String,
}

#[derive(Clone, Default)]
struct PartialMatch {
    chain: Vec<MatchedRoute>,
    params: Params,
    rank: Vec<u8>,
    /// The full pattern consumed so far, including layouts that have no
    /// component and therefore do not appear in `chain`.
    path: String,
}

#[derive(Clone)]
struct Candidate {
    chain: Vec<MatchedRoute>,
    rank: Vec<u8>,
}

/// Matches `pathname` against the route tree and returns the most specific
/// chain.
///
/// The result is the whole chain from the root route to the deepest match;
/// each entry is rendered in turn, and an [`crate::Outlet`] renders the next
/// one. An empty result means no route matched.
pub(crate) fn match_routes(root: &Route, pathname: &str) -> Vec<MatchedRoute> {
    let segments: Vec<&str> = pathname
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();

    match_route(root, &segments, 0, &PartialMatch::default())
        .map_or_else(Vec::new, |candidate| candidate.chain)
}

fn match_route(
    route: &Route,
    segments: &[&str],
    start: usize,
    parent: &PartialMatch,
) -> Option<Candidate> {
    let mut partial = PartialMatch {
        chain: parent.chain.clone(),
        params: parent.params.clone(),
        rank: parent.rank.clone(),
        path: parent.path.clone(),
    };

    let end = if route.is_index() {
        if start != segments.len() {
            return None;
        }
        start
    } else {
        let (end, own_params, own_rank) = route.pattern().consume(segments, start)?;
        partial.params.merge(&own_params);
        partial.rank.extend(own_rank);
        partial.path = join_paths(&partial.path, route.path());
        end
    };

    let current_match = route.component().map(|component| MatchedRoute {
        component: component.clone(),
        params: partial.params.clone(),
        path: partial.path.clone(),
    });

    let mut best = None;

    if end == segments.len() {
        if let Some(current_match) = &current_match {
            let mut candidate = Candidate {
                chain: partial.chain.clone(),
                rank: partial.rank.clone(),
            };
            candidate.rank.push(RANK_END);
            candidate.chain.push(current_match.clone());
            best = Some(candidate);
        }
    }

    // A child sees the parent in its chain, and its parameters, whether or not
    // the parent had a component of its own.
    if let Some(current_match) = current_match {
        partial.chain.push(current_match);
    }

    for child in route.child_routes() {
        if let Some(candidate) = match_route(child, segments, end, &partial) {
            if best
                .as_ref()
                .is_none_or(|current| is_better(&candidate, current))
            {
                best = Some(candidate);
            }
        }
    }

    best
}

fn is_better(candidate: &Candidate, current: &Candidate) -> bool {
    match candidate.rank.cmp(&current.rank) {
        Ordering::Greater => true,
        Ordering::Less => false,
        // Same pattern specificity: prefer the deeper chain, which is what
        // makes an index route render under its layout instead of the layout
        // alone with an empty outlet.
        Ordering::Equal => candidate.chain.len() > current.chain.len(),
    }
}

fn join_paths(parent: &str, child: &str) -> String {
    let child = child.trim_matches('/');
    if child.is_empty() {
        return if parent.is_empty() {
            "/".to_string()
        } else {
            parent.to_string()
        };
    }
    if parent.is_empty() || parent == "/" {
        format!("/{child}")
    } else {
        format!("{}/{child}", parent.trim_end_matches('/'))
    }
}

#[cfg(test)]
mod tests {
    use super::match_routes;
    use crate::{Route, index, layout, route};

    fn component() -> xs_primitive::Div {
        xs_primitive::div()
    }

    fn paths(root: &Route, pathname: &str) -> Vec<String> {
        match_routes(root, pathname)
            .into_iter()
            .map(|matched| matched.path)
            .collect()
    }

    #[test]
    fn a_static_route_matches_its_path() {
        let root = layout("")
            .child(route("/", component))
            .child(route("/about", component));
        assert_eq!(paths(&root, "/"), vec!["/"]);
        assert_eq!(paths(&root, "/about"), vec!["/about"]);
        assert!(paths(&root, "/missing").is_empty());
    }

    #[test]
    fn a_parameter_is_captured() {
        let root = route("/users/:id", component);
        let matched = match_routes(&root, "/users/42");
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].params.param("id"), "42");
        assert_eq!(matched[0].path, "/users/:id");
    }

    #[test]
    fn a_splat_captures_the_remainder() {
        let root = route("/files/*", component);
        let matched = match_routes(&root, "/files/a/b/c.txt");
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].params.param("*"), "a/b/c.txt");
    }

    #[test]
    fn a_static_segment_beats_a_parameter() {
        let root = layout("")
            .child(route("/users/:id", component))
            .child(route("/users/me", component));
        let matched = match_routes(&root, "/users/me");
        assert_eq!(matched[0].path, "/users/me");
    }

    #[test]
    fn a_parameter_beats_a_splat() {
        let root = layout("")
            .child(route("/files/*", component))
            .child(route("/files/:name", component));
        let matched = match_routes(&root, "/files/report");
        assert_eq!(matched[0].path, "/files/:name");
    }

    #[test]
    fn a_root_route_beats_a_zero_length_splat() {
        let root = layout("")
            .child(route("/", component))
            .child(route("*", component));
        let matched = match_routes(&root, "/");
        assert_eq!(matched[0].path, "/");
    }

    #[test]
    fn the_more_specific_of_two_parameters_wins() {
        let root = layout("")
            .child(route("/:section/:id", component))
            .child(route("/users/:id", component));
        let matched = match_routes(&root, "/users/42");
        assert_eq!(matched[0].path, "/users/:id");
    }

    #[test]
    fn query_and_fragment_do_not_affect_matching() {
        let root = route("/users", component);
        assert_eq!(paths(&root, "/users"), vec!["/users"]);
    }

    #[test]
    fn a_trailing_slash_matches_the_same_route() {
        let root = route("/users", component);
        assert_eq!(paths(&root, "/users/"), vec!["/users"]);
    }

    #[test]
    fn nested_routes_join_their_paths() {
        let root = route("/dashboard", component)
            .child(index(component))
            .child(route("settings", component))
            .child(route("users/:id", component));

        assert_eq!(paths(&root, "/dashboard"), vec!["/dashboard", "/dashboard"]);
        assert_eq!(
            paths(&root, "/dashboard/settings"),
            vec!["/dashboard", "/dashboard/settings"]
        );
        assert_eq!(
            paths(&root, "/dashboard/users/42"),
            vec!["/dashboard", "/dashboard/users/:id"]
        );
    }

    #[test]
    fn a_child_inherits_parent_parameters() {
        let root = route("/users/:user", component).child(route("posts/:post", component));
        let matched = match_routes(&root, "/users/42/posts/7");
        assert_eq!(matched.len(), 2);
        assert_eq!(matched[0].params.param("user"), "42");
        assert_eq!(matched[1].params.param("user"), "42");
        assert_eq!(matched[1].params.param("post"), "7");
    }

    #[test]
    fn an_index_route_is_preferred_over_its_layout_alone() {
        let root = route("/dashboard", component).child(index(component));
        assert_eq!(paths(&root, "/dashboard"), vec!["/dashboard", "/dashboard"]);
        assert_eq!(
            match_routes(&root, "/dashboard").len(),
            2,
            "the index route is the second node in the chain"
        );
    }

    #[test]
    fn a_layout_without_an_index_does_not_match_its_own_path() {
        let root = layout("/dashboard").child(route("settings", component));
        assert!(paths(&root, "/dashboard").is_empty());
        assert_eq!(
            paths(&root, "/dashboard/settings"),
            vec!["/dashboard/settings"]
        );
    }

    #[test]
    fn a_component_route_with_children_matches_its_own_path_even_without_an_index() {
        let root = route("/dashboard", component).child(route("settings", component));
        assert_eq!(paths(&root, "/dashboard"), vec!["/dashboard"]);
    }

    #[test]
    fn a_pathless_layout_wraps_a_child() {
        let root = layout("").child(route("/dashboard", component));
        assert_eq!(paths(&root, "/dashboard"), vec!["/dashboard"]);
    }

    #[test]
    fn a_pathless_layout_with_a_component_is_rendered() {
        let root = route("", component).child(route("/dashboard", component));
        assert_eq!(paths(&root, "/dashboard"), vec!["/", "/dashboard"]);
    }
}
