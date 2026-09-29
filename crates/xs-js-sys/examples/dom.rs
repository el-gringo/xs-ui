use std::cell::Cell;
use std::rc::Rc;

use xs_js_sys::{
    Closure, Element, EventEmitter, EventKeyDown, EventMouse, HTMLElement, Node, TokenList,
    document, log,
};

/// The demo state. Everything else lives in the dom: the rows, their text, the
/// done markers. Only what the dom cannot answer is kept here.
struct State {
    /// Id handed to the next todo, `todo-0`, `todo-1`, ...
    next_id: Cell<u32>,
    /// Number of rows in the list.
    total: Cell<u32>,
    /// Number of rows that are not done yet.
    remaining: Cell<u32>,
}

fn by_id(id: &str) -> HTMLElement {
    document().query_selector(&format!("#{id}"))
}

fn update_count(state: &State) {
    let total = state.total.get();
    let remaining = state.remaining.get();
    let text = if remaining == total {
        format!("{total} left")
    } else {
        format!("{remaining} left, {} done", total - remaining)
    };
    by_id("count").set_inner_html(&text);
}

/// One row: the text, and a delete button. Listeners are attached per row, so a
/// row rebuilt by `replace_child` comes with fresh ones.
fn todo_row(id: &str, text: &str, done: bool, state: &Rc<State>) -> HTMLElement {
    let row = document().create_element("li");
    row.set_attribute("id", id);
    row.set_attribute("data-done", if done { "true" } else { "false" });
    row.set_style("list-style", "none");
    row.set_style("margin", "6px 0");
    row.set_style("padding", "4px 8px");
    row.set_style("background", "#1e293b");
    row.set_style("border-radius", "6px");

    let label = document().create_element("span");
    label.set_attribute("id", &format!("{id}-label"));
    label.set_style("cursor", "pointer");
    label.set_style("user-select", "none");
    if done {
        label.class_list().add("done");
        label.set_style("opacity", "0.5");
        label.set_style("text-decoration", "line-through");
    }
    label.set_inner_html(text);
    row.append_child(&label);

    let remove = document().create_element("button");
    remove.set_attribute("id", &format!("{id}-remove"));
    remove.set_style("float", "right");
    remove.set_style("cursor", "pointer");
    remove.set_inner_html("&times;");
    row.append_child(&remove);

    // Clicking the label rebuilds the row, done or not, through replace_child.
    let toggled = id.to_string();
    let toggle_state = state.clone();
    label.add_event_listener(
        "click",
        Closure::new(move |_event: EventMouse| {
            let list = by_id("list");
            let old = by_id(&toggled);
            let text = by_id(&format!("{toggled}-label")).inner_html();
            let done = old.attribute("data-done").as_deref() == Some("true");

            list.replace_child(&todo_row(&toggled, &text, !done, &toggle_state), &old);
            if done {
                toggle_state.remaining.set(toggle_state.remaining.get() + 1);
            } else {
                toggle_state.remaining.set(toggle_state.remaining.get() - 1);
            }
            update_count(&toggle_state);
            log(&format!(
                "toggled #{toggled}, opacity is now {:?}",
                by_id(&format!("{toggled}-label")).style("opacity")
            ));
        }),
    );

    let deleted = id.to_string();
    let delete_state = state.clone();
    remove.add_event_listener(
        "click",
        Closure::new(move |_event: EventMouse| {
            let list = by_id("list");
            let item = by_id(&deleted);
            let done = item.attribute("data-done").as_deref() == Some("true");
            // The row is about to be detached, drop its state with it.
            item.remove_attribute("data-done");
            list.remove_child(&item);

            delete_state.total.set(delete_state.total.get() - 1);
            if !done {
                delete_state.remaining.set(delete_state.remaining.get() - 1);
            }
            update_count(&delete_state);
            log(&format!(
                "removed #{deleted}, {} left",
                delete_state.remaining.get()
            ));
        }),
    );

    row
}

fn add(state: &Rc<State>, text: &str) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }
    let id = format!("todo-{}", state.next_id.get());
    state.next_id.set(state.next_id.get() + 1);
    state.total.set(state.total.get() + 1);
    state.remaining.set(state.remaining.get() + 1);

    let item = todo_row(&id, text, false, state);
    let list = by_id("list");
    if state.total.get() == 1 {
        list.append_child(&item);
    } else {
        // Newest first.
        let first = list.query_selector("li");
        list.insert_before(&item, &first);
    }
    update_count(state);
    log(&format!("added #{id} {text:?}"));
}

pub fn main() {
    log("todo example: starting");
    let state = Rc::new(State {
        next_id: Cell::new(0),
        total: Cell::new(0),
        remaining: Cell::new(0),
    });

    let document = document();
    let body = document.query_selector("body");
    body.set_style("background", "#0f172a");
    body.set_style("color", "#e2e8f0");
    body.set_style("font-family", "sans-serif");
    body.set_style("padding", "24px");
    body.set_style("max-width", "520px");

    let title = document.create_element("h1");
    title.set_style("margin", "0 0 12px");
    title.set_inner_html("todo");
    body.append_child(&title);

    let input = document.create_element("input");
    input.set_attribute("id", "new-todo");
    input.set_attribute("placeholder", "what needs doing?");
    input.set_style("padding", "8px");
    input.set_style("width", "300px");
    body.append_child(&input);

    let add_button = document.create_element("button");
    add_button.set_attribute("id", "add");
    add_button.set_style("padding", "8px 12px");
    add_button.set_inner_html("add");
    body.append_child(&add_button);

    let list = document.create_element("ul");
    list.set_attribute("id", "list");
    list.set_style("padding", "0");
    body.append_child(&list);

    let count = document.create_element("p");
    count.set_attribute("id", "count");
    count.set_style("color", "#94a3b8");
    body.append_child(&count);

    // Enter in the input, or the button: take the value, clear the input, add.
    let input_state = state.clone();
    input.add_event_listener(
        "keydown",
        Closure::new(move |event: EventKeyDown| {
            if event.key() != "Enter" {
                return;
            }
            let input = by_id("new-todo");
            let text = input.value();
            input.set_value("");
            add(&input_state, &text);
        }),
    );
    let button_state = state.clone();
    add_button.add_event_listener(
        "click",
        Closure::new(move |_event: EventMouse| {
            let input = by_id("new-todo");
            let text = input.value();
            input.set_value("");
            add(&button_state, &text);
        }),
    );
    input.focus();

    add(&state, "write the todo list");
    add(&state, "port the bindings");
    add(&state, "ship it");
}
