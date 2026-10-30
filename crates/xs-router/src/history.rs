use std::{cell::RefCell, rc::Rc};

use crate::Location;

/// A navigation listener. Called with the location after every navigation.
pub(crate) type Listener = Rc<dyn Fn(Location)>;

/// The navigation stack the router reads and writes.
///
/// Implementations own their storage, so `push` and friends take `&self`; the
/// router keeps one boxed history for the lifetime of the app. `listen` is how
/// a history reports a change that did not originate in the router - the back
/// button on the web - as well as a change that did.
pub trait History {
    /// The current entry.
    fn location(&self) -> Location;

    /// Navigates to `to`, adding an entry after the current one.
    fn push(&self, to: &str);

    /// Navigates to `to`, replacing the current entry.
    fn replace(&self, to: &str);

    /// Moves one entry back.
    fn back(&self);

    /// Moves one entry forward.
    fn forward(&self);

    /// Moves `delta` entries. Negative goes back, positive goes forward, and
    /// a delta that would leave the stack is clamped to the nearest end.
    fn go(&self, delta: i32);

    /// Registers a listener for every subsequent navigation.
    fn listen(&self, listener: Listener);
}

impl<T: History + ?Sized> History for Box<T> {
    fn location(&self) -> Location {
        (**self).location()
    }

    fn push(&self, to: &str) {
        (**self).push(to);
    }

    fn replace(&self, to: &str) {
        (**self).replace(to);
    }

    fn back(&self) {
        (**self).back();
    }

    fn forward(&self) {
        (**self).forward();
    }

    fn go(&self, delta: i32) {
        (**self).go(delta);
    }

    fn listen(&self, listener: Listener) {
        (**self).listen(listener);
    }
}

struct MemoryState {
    entries: Vec<Location>,
    index: usize,
    listeners: Vec<Listener>,
}

/// An in-memory history.
///
/// This is what the router uses on native, where there is no browser stack.
/// It is also how tests drive the router without a display or a webview.
pub struct MemoryHistory {
    state: RefCell<MemoryState>,
}

impl MemoryHistory {
    #[must_use]
    pub fn new(initial: impl Into<String>) -> Self {
        Self {
            state: RefCell::new(MemoryState {
                entries: vec![Location::new(initial.into())],
                index: 0,
                listeners: Vec::new(),
            }),
        }
    }

    /// The whole stack, oldest first. Useful in tests and for debug output.
    #[must_use]
    pub fn entries(&self) -> Vec<Location> {
        self.state.borrow().entries.clone()
    }

    /// The current index in [`Self::entries`].
    #[must_use]
    pub fn index(&self) -> usize {
        self.state.borrow().index
    }
}

impl Default for MemoryHistory {
    fn default() -> Self {
        Self::new("/")
    }
}

impl History for MemoryHistory {
    fn location(&self) -> Location {
        let state = self.state.borrow();
        state.entries[state.index].clone()
    }

    fn push(&self, to: &str) {
        let mut state = self.state.borrow_mut();
        let index = state.index;
        let target = state.entries[index].resolve(to);
        state.entries.truncate(index + 1);
        state.entries.push(target.clone());
        state.index = state.entries.len() - 1;
        let listeners = state.listeners.clone();
        drop(state);
        notify(&listeners, &target);
    }

    fn replace(&self, to: &str) {
        let mut state = self.state.borrow_mut();
        let index = state.index;
        let target = state.entries[index].resolve(to);
        state.entries[index] = target.clone();
        let listeners = state.listeners.clone();
        drop(state);
        notify(&listeners, &target);
    }

    fn back(&self) {
        self.go(-1);
    }

    fn forward(&self) {
        self.go(1);
    }

    fn go(&self, delta: i32) {
        let mut state = self.state.borrow_mut();
        if state.entries.is_empty() {
            return;
        }
        let last = state.entries.len() as i32 - 1;
        let target = (state.index as i32 + delta).clamp(0, last) as usize;
        if target == state.index {
            return;
        }
        state.index = target;
        let location = state.entries[target].clone();
        let listeners = state.listeners.clone();
        drop(state);
        notify(&listeners, &location);
    }

    fn listen(&self, listener: Listener) {
        self.state.borrow_mut().listeners.push(listener);
    }
}

fn notify(listeners: &[Listener], location: &Location) {
    for listener in listeners {
        listener(location.clone());
    }
}

/// The real browser history.
///
/// `push` and `replace` call `history.pushState`/`replaceState` and then read
/// the location back from the browser, which is what resolves a relative url
/// exactly like the address bar would. `back`, `forward` and `go` let the
/// browser move and report back through `popstate`; updating the location
/// optimistically here would show the old page until the event arrived, and
/// would be wrong when the stack end is already where the user is.
#[cfg(target_arch = "wasm32")]
pub struct BrowserHistory {
    listeners: Rc<RefCell<Vec<Listener>>>,
}

#[cfg(target_arch = "wasm32")]
impl BrowserHistory {
    #[must_use]
    pub fn new() -> Self {
        use xs_js_sys::{Closure, Event, EventEmitter, window};

        let listeners: Rc<RefCell<Vec<Listener>>> = Rc::new(RefCell::new(Vec::new()));
        let watchers = Rc::clone(&listeners);
        // The binding table owns the closure for the lifetime of the page,
        // which is exactly what a popstate listener wants.
        window().add_event_listener(
            "popstate",
            Closure::new(move |_event: Event| {
                let location = Location::parse(&xs_js_sys::location_path());
                let listeners = watchers.borrow().clone();
                for listener in listeners {
                    listener(location.clone());
                }
            }),
        );

        Self { listeners }
    }

    fn notify(&self) {
        let location = Location::parse(&xs_js_sys::location_path());
        let listeners = self.listeners.borrow().clone();
        for listener in listeners {
            listener(location.clone());
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Default for BrowserHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "wasm32")]
impl History for BrowserHistory {
    fn location(&self) -> Location {
        Location::parse(&xs_js_sys::location_path())
    }

    fn push(&self, to: &str) {
        xs_js_sys::history_push_state(to);
        self.notify();
    }

    fn replace(&self, to: &str) {
        xs_js_sys::history_replace_state(to);
        self.notify();
    }

    fn back(&self) {
        xs_js_sys::history_back();
    }

    fn forward(&self) {
        xs_js_sys::history_forward();
    }

    fn go(&self, delta: i32) {
        xs_js_sys::history_go(delta);
    }

    fn listen(&self, listener: Listener) {
        self.listeners.borrow_mut().push(listener);
    }
}

pub(crate) fn platform_history() -> Box<dyn History> {
    #[cfg(target_arch = "wasm32")]
    {
        Box::new(BrowserHistory::new())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::new(MemoryHistory::default())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::{History, MemoryHistory};
    use crate::Location;

    #[test]
    fn push_truncates_the_forward_entries() {
        let history = MemoryHistory::new("/");
        history.push("/a");
        history.push("/b");
        history.back();
        assert_eq!(history.location().pathname, "/a");

        history.push("/c");
        assert_eq!(
            history
                .entries()
                .iter()
                .map(|location| location.pathname.as_str())
                .collect::<Vec<_>>(),
            vec!["/", "/a", "/c"]
        );
        assert_eq!(history.index(), 2);
    }

    #[test]
    fn replace_keeps_the_stack_length() {
        let history = MemoryHistory::new("/");
        history.push("/a");
        history.replace("/b");
        assert_eq!(
            history
                .entries()
                .iter()
                .map(|location| location.pathname.as_str())
                .collect::<Vec<_>>(),
            vec!["/", "/b"]
        );
        assert_eq!(history.index(), 1);
    }

    #[test]
    fn back_and_forward_move_the_index() {
        let history = MemoryHistory::new("/");
        history.push("/a");
        history.push("/b");

        history.back();
        assert_eq!(history.location().pathname, "/a");
        history.back();
        assert_eq!(history.location().pathname, "/");
        history.back();
        assert_eq!(
            history.location().pathname,
            "/",
            "the first entry is the end"
        );

        history.forward();
        assert_eq!(history.location().pathname, "/a");
        history.forward();
        assert_eq!(history.location().pathname, "/b");
        history.forward();
        assert_eq!(
            history.location().pathname,
            "/b",
            "the last entry is the end"
        );
    }

    #[test]
    fn go_clamps_out_of_range_deltas() {
        let history = MemoryHistory::new("/");
        history.push("/a");
        history.go(100);
        assert_eq!(history.location().pathname, "/a");
        history.go(-100);
        assert_eq!(history.location().pathname, "/");
    }

    #[test]
    fn listeners_see_every_navigation() {
        let history = MemoryHistory::new("/");
        let seen = Rc::new(RefCell::new(Vec::new()));
        let recorder = Rc::clone(&seen);
        history.listen(Rc::new(move |location| {
            recorder.borrow_mut().push(location.pathname);
        }));

        history.push("/a");
        history.replace("/b");
        history.back();
        history.forward();

        assert_eq!(
            *seen.borrow(),
            vec![
                "/a".to_string(),
                "/b".to_string(),
                "/".to_string(),
                "/b".to_string()
            ]
        );
    }

    #[test]
    fn a_relative_push_resolves_against_the_current_location() {
        let history = MemoryHistory::new("/users/42");
        history.push("posts");
        assert_eq!(history.location().pathname, "/users/posts");
    }

    #[test]
    fn the_location_is_a_parsed_url() {
        let history = MemoryHistory::new("/users?tab=posts#top");
        assert_eq!(history.location(), Location::parse("/users?tab=posts#top"));
    }
}
