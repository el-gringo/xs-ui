use xs_core::Undefined;

mod my_node {
    use super::*;

    #[allow(unused)]
    #[allow(nonstandard_style)]
    pub struct Props<name = Undefined, children = Undefined> {
        pub name: name,
        pub children: children,
    }

    #[allow(dead_code)]
    pub fn render(Props { name, children }: Props<&'_ str, Vec<String>>) -> String {
        format!("{name}({})", children.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xs_macro::rsx;

    #[test]
    fn test_rsx() {
        let result = rsx! {
            <my_node name="plouf"></my_custom_node>
        };
        assert_eq!(result, "plouf()");
    }

    #[test]
    fn test_rsx_with_children() {
        let result = rsx! {
            <my_node name="parent">
                <my_node name="child1"></my_node>
                <my_node name="child2">
                    <my_node name="child2_1"></my_node>
                </my_node>
            </my_node>
        };
        assert_eq!(result, "parent(child1(), child2(child2_1()))");
    }
}
