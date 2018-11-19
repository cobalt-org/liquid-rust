use super::Scalar;

/// Path to a value in an `Object`.
pub type Path = Vec<Scalar>;

/// Path to a value in an `Object`.
pub type PathRef<'s> = &'s [Scalar];
