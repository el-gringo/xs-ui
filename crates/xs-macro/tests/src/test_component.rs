#[cfg(test)]
mod test {
    use xs_macro::component;

    #[test]
    fn test_simple_component() {
        #[component]
        fn my_component() -> i32 {
            1 + 2
        }

        let result = my_component::render(my_component::Props {});
        assert_eq!(result, 3);
    }

    #[test]
    fn test_simple_component_with_arguments() {
        #[component]
        fn my_component(a: i32, b: i32) -> i32 {
            a + b + 2
        }

        let result = my_component::render(my_component::Props { a: 5, b: 10 });
        assert_eq!(result, 17);
    }

    #[test]
    fn test_simple_component_with_default_values() {
        #[component]
        fn my_component(a: i32, b: i32) -> i32 {
            a + b + 2
        }

        let result = my_component::render(my_component::Props { a: 5, b: 10 });
        assert_eq!(result, 17);
    }
}
