use xs_layout::{colors::{BLUE, GREEN, RED}, *};

#[allow(dead_code)]
pub fn sidebar() -> View {
    let border_color = Color::rgba(0.27, 0.27, 0.27, 255.0);
    let color_grey1 = Color::rgba(0.09, 0.09, 0.09, 255.0);
    let color_grey2 = Color::rgba(0.11, 0.11, 0.11, 255.0);
    let color_grey3 = Color::rgba(0.13, 0.13, 0.13, 255.0);
    let color_grey4 = Color::rgba(0.24, 0.24, 0.24, 255.0);
    let mut files: Vec<View> = vec![];
    let mut files2: Vec<View> = vec![];
    for _ in 0..10 {
        files.push(Box::new(Container {
            height: Some(20.0),
            width: None,
            color: color_grey4,
            margin: EdgeInsets::bottom(5.0),
            ..Default::default()
        }));
        files2.push(Box::new(Container {
            height: Some(25.0),
            width: None,
            color: color_grey4,
            margin: EdgeInsets::bottom(5.0),
            ..Default::default()
        }));
    }
    let widgets = Container {
        borders: Borders::bottom(Color::rgba(0.08, 0.39, 225.0, 0.89), 10.0),
        child: Some(Box::new(Flex {
            axis: Axis::Row,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Stretch,
            children: vec![
                Box::new(Container {
                    width: Some(50.0),
                    height: Some(f32::INFINITY),
                    color: color_grey2,
                    padding: EdgeInsets {
                        top: 6.0,
                        bottom: 0.0,
                        right: 6.0,
                        left: 6.0,
                    },
                    child: Some(Box::new(Flex {
                        axis: Axis::Column,
                        main_axis_size: MainAxisSize::Max,
                        main_axis_alignment: MainAxisAlignment::Start,
                        cross_axis_alignment: CrossAxisAlignment::Center,
                        children: vec![
                            Box::new(Container {
                                height: Some(40.0),
                                width: Some(40.0),
                                color: color_grey4,
                                margin: EdgeInsets::bottom(5.0),
                                ..Default::default()
                            }),
                            Box::new(Container {
                                height: Some(40.0),
                                width: Some(40.0),
                                color: color_grey4,
                                margin: EdgeInsets::bottom(5.0),
                                ..Default::default()
                            }),
                            Box::new(Container {
                                height: Some(40.0),
                                width: Some(40.0),
                                color: color_grey4,
                                margin: EdgeInsets::bottom(5.0),
                                ..Default::default()
                            }),
                            Box::new(Flexible {
                                flex_factor: 1.0,
                                child: Box::new(Spacer {}),
                            }),
                            Box::new(Container {
                                height: Some(40.0),
                                width: Some(40.0),
                                color: color_grey4,
                                margin: EdgeInsets::bottom(5.0),
                                ..Default::default()
                            }),
                        ],
                    })),
                    ..Default::default()
                }),
                Box::new(Container {
                    borders: Borders::right(border_color, 1.0),
                    height: Some(f32::INFINITY),
                    width: Some(200.0),
                    color: color_grey3,
                    child: Some(Box::new(Flex {
                        axis: Axis::Column,
                        main_axis_size: MainAxisSize::Max,
                        main_axis_alignment: MainAxisAlignment::Start,
                        cross_axis_alignment: CrossAxisAlignment::Stretch,
                        children: files,
                    })),
                    ..Default::default()
                }),
                Box::new(Flexible {
                    flex_factor: 1.0,
                    child: Box::new(Container {
                        color: color_grey1,
                        alignment: Alignment::center(),
                        child: Some(Box::new(Flex {
                            axis: Axis::Column,
                            main_axis_size: MainAxisSize::Min,
                            main_axis_alignment: MainAxisAlignment::Start,
                            cross_axis_alignment: CrossAxisAlignment::Center,
                            children: vec![
                                Box::new(Container {
                                    width: Some(150.0),
                                    height: Some(15.0),
                                    margin: EdgeInsets::bottom(5.0),
                                    color: color_grey4,
                                    ..Default::default()
                                }),
                                Box::new(Container {
                                    width: Some(100.0),
                                    height: Some(15.0),
                                    margin: EdgeInsets::bottom(5.0),
                                    color: color_grey3,
                                    ..Default::default()
                                }),
                                Box::new(Container {
                                    width: Some(150.0),
                                    height: Some(15.0),
                                    margin: EdgeInsets::bottom(5.0),
                                    color: color_grey4,
                                    ..Default::default()
                                }),
                            ],
                        })),
                        ..Default::default()
                    }),
                }),
                Box::new(Container {
                    width: Some(175.0),
                    height: Some(f32::INFINITY),
                    color: color_grey3,
                    borders: Borders::left(border_color, 1.0),
                    child: Some(Box::new(Flex {
                        axis: Axis::Column,
                        main_axis_size: MainAxisSize::Max,
                        main_axis_alignment: MainAxisAlignment::Start,
                        cross_axis_alignment: CrossAxisAlignment::Stretch,
                        children: files2,
                    })),
                    ..Default::default()
                }),
            ],
        })),
        ..Container::default()
    };
    Box::new(widgets)
}

#[allow(dead_code)]
pub fn render_nested_positioned() -> View {
    Box::new(Positioned {
        position: (100.0, 100.0).into(),
        child: Box::new(Container {
            color: BLUE,
            width: Some(400.0),
            height: Some(400.0),
            child: Some(
                Box::new(Positioned {
                    position: (100.0, 0.0).into(),
                    child: Box::new(Container {
                            color: GREEN,
                            width: Some(100.0),
                            height: Some(100.0),
                            ..Default::default()
                    }),
                }),
            ),
            ..Default::default()
        }),
    })
}

#[allow(dead_code)]
pub fn flex() -> View {

    Box::new(Positioned {
        position: (0.0, 0.0).into(),
        child: Box::new(Container {
            width: Some(320.),
            color: RED,
            child: Some(Box::new(Flex {
                axis: Axis::Column,
                main_axis_alignment: MainAxisAlignment::Start,
                cross_axis_alignment: CrossAxisAlignment::Center,
                main_axis_size: MainAxisSize::Min,
                children: vec![
                    Box::new(Container {
                        width: Some(200.),
                        height: Some(40.),
                        color: BLUE,
                        margin: EdgeInsets { top: 8., bottom: 8., left: 16., right: 16. },
                        ..Container::default()
                    }),

                    // Box::new(Container {
                    //     width: Some(200.),
                    //     height: Some(40.),
                    //     color: GREEN,
                    //     margin: EdgeInsets { top: 16., bottom: 0., left: 16., right: 16. },
                    //     ..Container::default()
                    // }),
                    // Box::new(Container {
                    //     width: Some(200.),
                    //     height: Some(40.),
                    //     color: YELLOW,
                    //     margin: EdgeInsets { top: 16., bottom: 0., left: 16., right: 16. },
                    //     ..Container::default()
                    // })
                ]
            })),
            ..Container::default()
        })
    })
}
