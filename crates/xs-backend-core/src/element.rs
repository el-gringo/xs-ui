use std::rc::Rc;

use crate::{AppContext, Render};
use xs_layout::View;

/*/
pub struct Element {
    paint: dyn Fn(&mut Element, &mut AppContext) -> View,
}

/
impl Element {
    pub fn mount(self, ctx: &mut AppContext) {
        let node = Node::new(self);
        ctx.current_node = Some(node);
        ctx.nodes.insert(, v)
    }
}
*/

pub trait Element {
    fn children(&self) -> Vec<Rc<Box<dyn Render>>> {
        Vec::new()
    }

    fn paint(&self, ctx: &mut AppContext, views: Vec<View>) -> View;
}

/*/
impl From<Box<dyn Element>> for Node {
    fn from(value: Box<dyn Element>) -> Self {
        Node::new(value)
    }
}
*/
