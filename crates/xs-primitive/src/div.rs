use xs_core::Undefined;
use xs_layout::{Container, Flex, Layout, Positioned};

use crate::style::{Style, StyleDisplay, StylePosition};

#[allow(non_camel_case_types)]
pub struct Props<style = Undefined, children = Undefined> {
    #[allow(unused)]
    pub style: style,
    #[allow(unused)]
    pub children: children,
}

pub fn render(props: Props<Style, Vec<Box<dyn Layout>>>) -> Box<dyn Layout> {
    let mut style = props.style;
    let children = props.children;

    style.prepare();
    let child: Option<Box<dyn Layout>> = if children.len() > 0 {
        Some(match style.display {
            StyleDisplay::Flex => Box::new(Flex {
                axis: style.flex_direction,
                // main_axis_size: if style.flex_grow {
                //     MainAxisSize::Max
                // } else {
                //     MainAxisSize::Min
                // },
                main_axis_alignment: style.justify_content,
                cross_axis_alignment: style.align_items,
                children,
                ..Flex::default()
            }),
            StyleDisplay::Static => todo!(),
            StyleDisplay::Inline => todo!(),
        })
    } else {
        None
    };
    let container = Box::new(Container {
        color: style.background_color,
        width: style.width,
        height: style.height,
        margin: style.margin,
        borders: style.border,
        padding: style.padding,
        child,
        ..Container::default()
    });
    let result: Box<dyn Layout> = match style.position {
        StylePosition::Absolute => Box::new(Positioned {
            position: (style.left.unwrap_or(0.), style.top.unwrap_or(0.)).into(),
            child: container,
        }),
        StylePosition::Static => container,
    };
    result
}

#[cfg(test)]
mod test {
    use xs_layout::{
        BoxConstraints, Container, Layout, LayoutTree, Positioned,
        colors::{BLUE, RED, YELLOW},
    };

    use crate::{
        Props,
        div::render,
        style::{Style, StylePosition},
    };

    const WIDTH: f32 = 1920.;
    const HEIGH: f32 = 1080.;

    #[test]
    fn test_div_absolute() {
        let node1 = render(Props {
            style: Style {
                position: StylePosition::Absolute,
                left: Some(0.),
                top: Some(0.),
                width: Some(WIDTH),
                height: Some(HEIGH),
                background_color: RED,
                ..Style::default()
            },
            children: vec![render(Props {
                style: Style {
                    position: StylePosition::Absolute,
                    left: Some(100.),
                    top: Some(100.),
                    width: Some(100.),
                    height: Some(100.),
                    background_color: BLUE,
                    ..Style::default()
                },
                children: vec![],
            })],
        });
        let constraints = BoxConstraints {
            min: (0.0, 0.0).into(),
            max: (WIDTH, HEIGH).into(),
        };
        let mut tree1 = LayoutTree::new();
        node1.layout(&mut tree1, &constraints);

        let node2 = Box::new(Positioned {
            position: (0., 0.).into(),
            child: Box::new(Container {
                width: Some(WIDTH),
                height: Some(HEIGH),
                color: RED,
                child: Some(Box::new(Positioned {
                    position: (100., 100.).into(),
                    child: Box::new(Container {
                        width: Some(100.),
                        height: Some(100.),
                        color: BLUE,
                        ..Container::default()
                    }),
                })),
                ..Container::default() // child: Some(Box::new(Flex {
                                       //     axis: Axis::Column,
                                       //     main_axis_size: MainAxisSize::Max,
                                       //     main_axis_alignment: MainAxisAlignment::Start,
                                       //     cross_axis_alignment: CrossAxisAlignment::Center,
                                       //     children: vec![],
                                       // })),
            }),
        });

        let mut tree2 = LayoutTree::new();
        node2.layout(&mut tree2, &constraints);
        pretty_assertions::assert_eq!(format!("{tree1:#?}"), format!("{tree2:#?}"));
    }
}
