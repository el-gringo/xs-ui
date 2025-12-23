use xs_core::Undefined;
use xs_macros::rsx;

mod my_custom_node {
    use super::*;

    #[allow(unused)]
    #[allow(nonstandard_style)]
    pub struct Props<class=Undefined> {
        class: class,
    }

    pub fn render(Props { class }: Props<&'_ str>) -> &'_ str {
        class
    }
}

#[test]
fn test_rsx() {

    let node = rsx! {
        <my_custom_node class="plouf"></my_custom_node>
    };
}
