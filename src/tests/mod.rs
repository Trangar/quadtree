#![cfg(test)]

use crate::{IdentityPoint, Point};

mod find_range;
mod insert;
mod remove;
mod update;

/// Helper function to generate an [`IdentityPoint`]
fn ip(identity: u32, x: f32, y: f32) -> IdentityPoint {
    IdentityPoint {
        identity,
        point: Point::new(x, y),
    }
}

/// Helper function to generate a tuple of [`IdentityPoint`] and value `V`
fn ipv<V>(identity: u32, x: f32, y: f32, value: V) -> (IdentityPoint, V) {
    (
        IdentityPoint {
            identity,
            point: Point::new(x, y),
        },
        value,
    )
}
