#![allow(clippy::cast_precision_loss)]

use crate::{tests::ip, Point, QuadTree};
use noisy_float::types::r32;

#[test]
pub fn in_range() {
    let mut tree = QuadTree::<u32, u32, 4>::sized_around_origin(Point::new(10., 10.));

    let mut n = 0;
    // Create a cross of points around 0,0
    for i in -10..=10 {
        tree.insert(ip(n, i as f32, i as f32), n);
        n += 1;
    }
    for i in -10..=-1 {
        tree.insert(ip(n, i as f32, -(i as f32)), n);
        n += 1;
    }
    for i in 1..=10 {
        tree.insert(ip(n, i as f32, -(i as f32)), n);
        n += 1;
    }

    let mut points = Vec::new();
    tree.find_range(Point::zero(), r32(3.5), |id, point, v| {
        points.push((*id, point.x, point.y, *v))
    });
    assert_eq!(
        points,
        vec![
            (8, r32(-2.0), r32(-2.0), 8),
            (9, -r32(1.0), r32(-1.0), 9),
            (31, r32(1.0), r32(-1.0), 31),
            (32, r32(2.0), r32(-2.0), 32),
            (29, r32(-2.0), r32(2.0), 29),
            (30, r32(-1.0), r32(1.0), 30),
            (10, r32(0.0), r32(0.0), 10),
            (11, r32(1.0), r32(1.0), 11),
            (12, r32(2.0), r32(2.0), 12),
        ]
    );

    let mut points = Vec::new();
    tree.find_range(Point::new(4.5, 4.5), r32(5.5), |id, point, v| {
        points.push((*id, point.x, point.y, *v))
    });
    assert_eq!(
        points,
        vec![
            (11, r32(1.0), r32(1.0), 11),
            (12, r32(2.0), r32(2.0), 12),
            (13, r32(3.0), r32(3.0), 13),
            (14, r32(4.0), r32(4.0), 14),
            (15, r32(5.0), r32(5.0), 15),
            (16, r32(6.0), r32(6.0), 16),
            (17, r32(7.0), r32(7.0), 17),
            (18, r32(8.0), r32(8.0), 18),
        ]
    );
}

#[test]
pub fn out_of_range() {
    let mut tree = QuadTree::<u32, u32, 4>::sized_around_origin(Point::new(10., 10.));

    for (n, i) in (-15..=15).enumerate() {
        let n = u32::try_from(n).unwrap();
        tree.insert(ip(n, i as f32, i as f32), n);
    }

    let mut points = Vec::new();
    tree.find_range(Point::new(-10., -10.), r32(3.5), |id, point, v| {
        points.push((*id, point.x, point.y, *v))
    });
    assert_eq!(
        points,
        vec![
            (5, r32(-10.0), r32(-10.0), 5),
            (6, r32(-9.0), r32(-9.0), 6),
            (7, r32(-8.0), r32(-8.0), 7),
            (3, r32(-12.0), r32(-12.0), 3),
            (4, r32(-11.0), r32(-11.0), 4),
        ]
    );
}
