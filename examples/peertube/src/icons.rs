use xs::Svg;

pub const ICON_SIZE: f32 = 28.;

#[must_use]
pub fn for_category(id: u32) -> Option<&'static Svg> {
    if id == 0 {
        return None;
    }
    let index: usize = id.try_into().ok()?;
    ICONS.get(index)
}

#[must_use]
pub fn settings() -> &'static Svg {
    &ICONS[0]
}

static ICONS: [Svg; 19] = [
    Svg::new(include_str!("../../../ressources/icons/settings.svg")),
    Svg::new(include_str!("../../../ressources/icons/music.svg")),
    Svg::new(include_str!("../../../ressources/icons/film.svg")),
    Svg::new(include_str!("../../../ressources/icons/car.svg")),
    Svg::new(include_str!("../../../ressources/icons/palette.svg")),
    Svg::new(include_str!("../../../ressources/icons/trophy.svg")),
    Svg::new(include_str!("../../../ressources/icons/plane.svg")),
    Svg::new(include_str!("../../../ressources/icons/gamepad-2.svg")),
    Svg::new(include_str!("../../../ressources/icons/users.svg")),
    Svg::new(include_str!("../../../ressources/icons/smile.svg")),
    Svg::new(include_str!("../../../ressources/icons/star.svg")),
    Svg::new(include_str!("../../../ressources/icons/newspaper.svg")),
    Svg::new(include_str!("../../../ressources/icons/wrench.svg")),
    Svg::new(include_str!("../../../ressources/icons/graduation-cap.svg")),
    Svg::new(include_str!("../../../ressources/icons/megaphone.svg")),
    Svg::new(include_str!("../../../ressources/icons/atom.svg")),
    Svg::new(include_str!("../../../ressources/icons/paw-print.svg")),
    Svg::new(include_str!("../../../ressources/icons/balloon.svg")),
    Svg::new(include_str!("../../../ressources/icons/utensils.svg")),
];
