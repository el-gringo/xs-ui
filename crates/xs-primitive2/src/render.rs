use xs_layout::{Container, Flex, Positioned, View};

use crate::{
    element::Element,
    style::{StyleDisplay, StylePosition},
};

pub trait Render {
    fn render(&mut self) -> View;
}

// pub type Renderable = Box<dyn Render>;

impl Render for Element {
    fn render(&mut self) -> View {
        let style = self.style;
        let mut children: Vec<View> = Vec::new();
        for child in self.children.iter_mut() {
            children.push(child.render());
        }
        let child: Option<View> = if children.len() > 0 {
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
        let result = match style.position {
            StylePosition::Absolute => Box::new(Positioned {
                position: (style.left.unwrap_or(0.), style.top.unwrap_or(0.)).into(),
                child: container,
            }),
            StylePosition::Static => container as View,
        };
        result
    }
}
