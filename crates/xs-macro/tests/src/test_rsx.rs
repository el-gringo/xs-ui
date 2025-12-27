#[cfg(test)]
mod tests {
    use xs_core::Undefined;
    use xs_macro::rsx;

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

    mod my_container {
        use super::*;

        #[allow(unused)]
        #[allow(nonstandard_style)]
        pub struct Props<name = Undefined, children = Undefined> {
            pub name: name,
            pub children: children,
        }

        #[allow(dead_code)]
        pub fn render(Props { name, children }: Props<&'_ str, Vec<String>>) -> String {
            rsx! { <my_node name={name}>{children}</my_node> }
            //my_node::render(my_node::Props { name: name, children })
        }
    }

    #[test]
    fn test_rsx() {
        let result = rsx! {
            <my_node name="plouf"></my_node>
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

    #[test]
    fn test_container() {
        let result = rsx! {
            <my_container name="container">
                <my_node name="child1"></my_node>
                <my_node name="child2">
                    <my_node name="child2_1"></my_node>
                </my_node>
            </my_container>
        };
        //assert_eq!(result, "");

        let result = my_container::render(my_container::Props {
            name: "container",
            children: vec![
                my_node::render(my_node::Props {
                    name: "child1",
                    children: vec![],
                }),
                my_node::render(my_node::Props {
                    name: "child2",
                    children: vec![my_node::render(my_node::Props {
                        name: "child2_1",
                        children: vec![],
                    })],
                }),
            ],
        });
        println!("result: {result}");
        assert_eq!(result, "");

    }
}
