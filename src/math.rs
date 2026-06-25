use core::{
    cmp::{max, min},
    ops::Add,
};

use agb::fixnum::{Rect, Vector2D, vec2};

/// Given an input, a min, and a max, clamps the input between the two values (inclusive)
#[inline]
pub fn clamp<T>(value: T, min_value: T, max_value: T) -> T
where
    T: Ord,
{
    min(max_value, max(min_value, value))
}

/// Given an input Vector2D and a bounding Rect, restrict the Vector2D to those bounds (inclusive)
#[inline]
pub fn bound<T>(vec: Vector2D<T>, bounds: Rect<T>) -> Vector2D<T>
where
    T: Copy,
    T: Ord,
    T: Add<T, Output = T>,
{
    vec2(
        clamp(vec.x, bounds.position.x, bounds.position.x + bounds.size.x),
        clamp(vec.y, bounds.position.y, bounds.position.y + bounds.size.y),
    )
}
