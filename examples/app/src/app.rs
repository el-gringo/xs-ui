use xs_layout::*;


#[allow(dead_code)]
pub fn positioned() -> Box<dyn Layout> {
    Box::new(Positioned {
        position: (0.0, 0.0).into(),
        child: Box::new(Container {
            color: Color::red(),
            width: Some(1920.0),
            height: Some(1080.0),
            child: Some(Box::new(Positioned {
                position: (100.0, 100.0).into(),
                child: Box::new(Positioned {
                    position: (200.0, 200.0).into(),
                    child: Box::new(Container {
                        borders: Borders::right(Color::blue(), 10.0),
                        color: Color::green(),
                        width: Some(100.0),
                        height: Some(100.0),
                        ..Default::default()
                    }),
                }),
            })),
            ..Default::default()
        })
    })
}

#[allow(dead_code)]
pub fn sidebar() -> Box<dyn Layout> {
    let border_color = Color::rgba(70.0, 70.0, 70.0, 255.0);
    let color_grey1 = Color::rgba(22.0, 22.0, 22.0, 255.0);
    let color_grey2 = Color::rgba(30.0, 30.0, 30.0, 255.0);
    let color_grey3 = Color::rgba(35.0, 35.0, 35.0, 255.0);
    let color_grey4 = Color::rgba(60.0, 60.0, 60.0, 255.0);
    let mut files: Vec<Box<dyn FlexLayout>> = vec![];
    let mut files2: Vec<Box<dyn FlexLayout>> = vec![];
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
        borders: Borders::bottom(Color::rgba(20.0, 100.0, 225.0, 255.0), 10.0),
        child: Some(Box::new(Flex {
            axis: Axis::Horizontal,
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
                        axis: Axis::Vertical,
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
                        axis: Axis::Vertical,
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
                            axis: Axis::Vertical,
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
                    color: Color::rgba(35.0, 35.0, 35.0, 255.0),
                    borders: Borders::left(border_color, 1.0),
                    child: Some(Box::new(Flex {
                        axis: Axis::Vertical,
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
