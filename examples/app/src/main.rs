#![allow(dead_code)]

use xs_backend::Backend;
use xs_layout::{
    Axis, BorderSide, Borders, Color, Container, CrossAxisAlignment, EdgeInsets, Layout, MainAxisAlignment, Positioned, colors::{BLUE, GREEN, RED, YELLOW}
};
// use xs_renderer_core::{
//     div,
//     style::{Style, StyleDisplay, StylePosition},
// };
mod app;

const WIDTH: f32 = 960.;
const HEIGH: f32 = 540.;

// pub fn div1() -> Box<dyn Layout> {
//     div::render(
//         &mut Style {
//             width: Some(300.),
//             background_color: Color::rgba(0.125, 0.125, 0.125, 255.),
//             display: StyleDisplay::Flex,
//             flex_direction: Axis::Column,
//             justify_content: MainAxisAlignment::Start,
//             align_items: CrossAxisAlignment::Center,
//             ..Style::default()
//         },
//         vec![
//             div::render(
//                 &mut Style {
//                     width: Some(200.),
//                     height: Some(40.),
//                     background_color: RED,
//                     margin: EdgeInsets {
//                         top: 8.,
//                         bottom: 0.,
//                         left: 16.,
//                         right: 16.,
//                     },
//                     border: Borders {
//                         top: Some(BorderSide { width: 2., color: YELLOW }),
//                         right: Some(BorderSide { width: 2., color: YELLOW }),
//                         bottom: Some(BorderSide { width: 2., color: YELLOW }),
//                         left: Some(BorderSide { width: 2., color: YELLOW }),
//                     },
//                     ..Style::default()
//                 },
//                 vec![],
//             ),
//             div::render(
//                 &mut Style {
//                     width: Some(200.),
//                     height: Some(40.),
//                     background_color: GREEN,
//                     margin: EdgeInsets {
//                         top: 8.,
//                         bottom: 0.,
//                         left: 16.,
//                         right: 16.,
//                     },
//                     border: Borders {
//                         top: Some(BorderSide { width: 2., color: YELLOW }),
//                         right: Some(BorderSide { width: 2., color: YELLOW }),
//                         bottom: Some(BorderSide { width: 2., color: YELLOW }),
//                         left: Some(BorderSide { width: 2., color: YELLOW }),
//                     },
//                     ..Style::default()
//                 },
//                 vec![],
//             ),
//             div::render(
//                 &mut Style {
//                     width: Some(200.),
//                     height: Some(40.),
//                     background_color: BLUE,
//                     margin: EdgeInsets {
//                         top: 8.,
//                         bottom: 0.,
//                         left: 16.,
//                         right: 16.,
//                     },
//                     border: Borders {
//                         top: Some(BorderSide { width: 2., color: YELLOW }),
//                         right: Some(BorderSide { width: 2., color: YELLOW }),
//                         bottom: Some(BorderSide { width: 2., color: YELLOW }),
//                         left: Some(BorderSide { width: 2., color: YELLOW }),
//                     },
//                     ..Style::default()
//                 },
//                 vec![],
//             ),
//         ],
//     )
// }

// pub fn div_absolute() -> Box<dyn Layout> {
//     div::render(
//         &mut Style {
//             position: StylePosition::Absolute,
//             left: Some(0.),
//             top: Some(0.),
//             width: Some(WIDTH),
//             height: Some(HEIGH),
//             background_color: RED,
//             ..Style::default()
//         },
//         vec![div::render(
//             &mut Style {
//                 position: StylePosition::Absolute,
//                 left: Some(100.),
//                 top: Some(100.),
//                 width: Some(100.),
//                 height: Some(100.),
//                 background_color: BLUE,
//                 ..Style::default()
//             },
//             vec![
//                 div::render(
//                     &mut Style {
//                         position: StylePosition::Absolute,
//                         left: Some(80.),
//                         top: Some(80.),
//                         width: Some(20.),
//                         height: Some(20.),
//                         background_color: YELLOW,
//                         ..Style::default()
//                     },
//                     vec![

//                     ],
//                 )
//             ],
//         )],
//     )
// }

fn raw_absolute() -> Box<dyn Layout> {
    Box::new(Container {
        width: Some(WIDTH),
        height: Some(HEIGH),
        color: RED,
        child: Some(Box::new(Positioned {
                position: (100., 100.).into(),
                child: Box::new(Container {
                    width: Some(100.),
                    height: Some(100.),
                    color: BLUE,
                    child: Some(
                        Box::new(Positioned {
                            position: (80., 80.).into(),
                            child: Box::new(Container {
                                width: Some(20.),
                                height: Some(20.),
                                color: YELLOW,
                                child: None,
                                ..Container::default()
                            })
                        })
                    ),
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
    })
}


pub fn main() {
    let mut backend = Backend::new(1280, 720);
    backend.log("test");
    backend.run(|| {
        app::render_nested_positioned()
    });
}
