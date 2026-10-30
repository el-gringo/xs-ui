use crate::Params;

/// Segment ranks, used to choose the most specific route when more than one
/// matches. The end sentinel makes an exact match beat a zero-length splat.
pub(crate) const RANK_STATIC: u8 = 3;
pub(crate) const RANK_PARAM: u8 = 2;
pub(crate) const RANK_SPLAT: u8 = 1;
pub(crate) const RANK_END: u8 = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Segment {
    Static(String),
    Param(String),
    Splat,
}

/// A route path compiled to its segments.
///
/// Only the last `*` is meaningful; anything after it is ignored, the same
/// way the splat itself consumes the rest of the location.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Pattern {
    segments: Vec<Segment>,
}

impl Pattern {
    pub(crate) fn new(path: &str) -> Self {
        let path = path.split(['?', '#']).next().unwrap_or_default();
        let mut segments = Vec::new();
        for segment in path.split('/').filter(|segment| !segment.is_empty()) {
            if let Some(name) = segment.strip_prefix(':') {
                if !name.is_empty() {
                    segments.push(Segment::Param(name.to_string()));
                    continue;
                }
            }
            if segment == "*" {
                segments.push(Segment::Splat);
                break;
            }
            segments.push(Segment::Static(segment.to_string()));
        }
        Self { segments }
    }

    /// Consumes `segments[start..]` against this pattern.
    ///
    /// Returns where the match ended, the parameters it captured, and one rank
    /// byte per consumed segment. A pattern that needs more segments than are
    /// left, or whose static segment differs, does not match.
    pub(crate) fn consume(
        &self,
        segments: &[&str],
        start: usize,
    ) -> Option<(usize, Params, Vec<u8>)> {
        let mut index = start;
        let mut params = Params::new();
        let mut rank = Vec::with_capacity(self.segments.len());

        for segment in &self.segments {
            match segment {
                Segment::Static(expected) => {
                    if *segments.get(index)? != expected.as_str() {
                        return None;
                    }
                    index += 1;
                    rank.push(RANK_STATIC);
                }
                Segment::Param(name) => {
                    let value = *segments.get(index)?;
                    params.insert(name.clone(), value);
                    index += 1;
                    rank.push(RANK_PARAM);
                }
                Segment::Splat => {
                    let rest = segments[index..].join("/");
                    params.insert("*", rest.clone());
                    params.insert("splat", rest);
                    index = segments.len();
                    rank.push(RANK_SPLAT);
                    break;
                }
            }
        }

        Some((index, params, rank))
    }
}

#[cfg(test)]
mod tests {
    use super::{Pattern, RANK_PARAM, RANK_SPLAT, RANK_STATIC, Segment};

    fn segments(path: &str) -> Vec<&str> {
        path.split('/')
            .filter(|segment| !segment.is_empty())
            .collect()
    }

    #[test]
    fn static_segments_only_match_themselves() {
        let pattern = Pattern::new("/users");
        let (end, params, rank) = pattern.consume(&segments("/users"), 0).unwrap();
        assert_eq!(end, 1);
        assert!(params.is_empty());
        assert_eq!(rank, vec![RANK_STATIC]);

        assert!(pattern.consume(&segments("/posts"), 0).is_none());
    }

    #[test]
    fn a_parameter_captures_one_segment() {
        let pattern = Pattern::new("/users/:id");
        let (end, params, rank) = pattern.consume(&segments("/users/42"), 0).unwrap();
        assert_eq!(end, 2);
        assert_eq!(params.param("id"), "42");
        assert_eq!(rank, vec![RANK_STATIC, RANK_PARAM]);
    }

    #[test]
    fn a_splat_captures_the_rest() {
        let pattern = Pattern::new("/files/*");
        let (end, params, rank) = pattern.consume(&segments("/files/a/b/c.txt"), 0).unwrap();
        assert_eq!(end, 4);
        assert_eq!(params.param("*"), "a/b/c.txt");
        assert_eq!(params.param("splat"), "a/b/c.txt");
        assert_eq!(rank, vec![RANK_STATIC, RANK_SPLAT]);
    }

    #[test]
    fn a_splat_matches_zero_segments() {
        let pattern = Pattern::new("/files/*");
        let (end, params, _) = pattern.consume(&segments("/files"), 0).unwrap();
        assert_eq!(end, 1);
        assert_eq!(params.param("*"), "");
    }

    #[test]
    fn query_and_fragment_are_not_part_of_the_pattern() {
        assert_eq!(Pattern::new("/users?tab=posts#top"), Pattern::new("/users"));
    }

    #[test]
    fn a_trailing_slash_does_not_create_an_empty_segment() {
        assert_eq!(Pattern::new("/users/"), Pattern::new("/users"));
        assert_eq!(
            Pattern::new("/users/").segments,
            vec![Segment::Static("users".to_string())]
        );
    }

    #[test]
    fn a_bare_colon_is_a_static_segment() {
        assert_eq!(
            Pattern::new("/:").segments,
            vec![Segment::Static(":".to_string())]
        );
    }
}
