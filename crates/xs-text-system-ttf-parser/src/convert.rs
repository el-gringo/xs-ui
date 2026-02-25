use crate::Face;

#[cfg(feature = "variable-fonts")]
use crate::Tag;

/// Used to perform a cheap conversion to a [`Face`](struct.Face.html) reference.
pub trait AsFaceRef {
    /// Convert to a [`Face`](struct.Face.html) reference.
    fn as_face_ref(&self) -> &Face<'_>;
}

impl AsFaceRef for Face<'_> {
    #[inline]
    fn as_face_ref(&self) -> &Face<'_> {
        self
    }
}

impl AsFaceRef for &Face<'_> {
    #[inline]
    fn as_face_ref(&self) -> &Face<'_> {
        self
    }
}

/// Trait exposing mutable operations on a [`ttf_parser::Face`].
pub trait FaceMut {
    /// Sets a variation axis coordinate.
    ///
    /// See [`ttf_parser::Face::set_variation`].
    #[cfg(feature = "variable-fonts")]
    fn set_variation(&mut self, axis: Tag, value: f32) -> Option<()>;
}
impl FaceMut for Face<'_> {
    #[inline]
    #[cfg(feature = "variable-fonts")]
    fn set_variation(&mut self, axis: Tag, value: f32) -> Option<()> {
        Face::set_variation(self.as_face_ref(), axis, value)
    }
}
impl FaceMut for &mut Face<'_> {
    #[inline]
    #[cfg(feature = "variable-fonts")]
    fn set_variation(&mut self, axis: Tag, value: f32) -> Option<()> {
        Face::set_variation(self, axis, value)
    }
}
