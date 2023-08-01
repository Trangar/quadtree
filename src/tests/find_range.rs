#![allow(clippy::cast_precision_loss)]

use crate::{tests::ip, Point, QuadTree};
use noisy_float::types::r32;

#[test]
pub fn in_range() {
    let mut tree = QuadTree::<u32, 4>::sized_around_origin(Point::new(10., 10.));

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
    tree.find_range(Point::zero(), r32(3.5), |p, v| points.push((p, v)));
    assert_eq!(
        points,
        vec![
            (ip(8, -2.0, -2.0), &8),
            (ip(9, -1.0, -1.0), &9),
            (ip(31, 1.0, -1.0), &31),
            (ip(32, 2.0, -2.0), &32),
            (ip(29, -2.0, 2.0), &29),
            (ip(30, -1.0, 1.0), &30),
            (ip(10, 0.0, 0.0), &10),
            (ip(11, 1.0, 1.0), &11),
            (ip(12, 2.0, 2.0), &12),
        ]
    );

    let mut points = Vec::new();
    tree.find_range(Point::new(4.5, 4.5), r32(5.5), |p, v| points.push((p, v)));
    assert_eq!(
        points,
        vec![
            (ip(11, 1.0, 1.0), &11),
            (ip(12, 2.0, 2.0), &12),
            (ip(13, 3.0, 3.0), &13),
            (ip(14, 4.0, 4.0), &14),
            (ip(15, 5.0, 5.0), &15),
            (ip(16, 6.0, 6.0), &16),
            (ip(17, 7.0, 7.0), &17),
            (ip(18, 8.0, 8.0), &18),
        ]
    );
}

#[test]
pub fn out_of_range() {
    let mut tree = QuadTree::<u32, 4>::sized_around_origin(Point::new(10., 10.));

    for (n, i) in (-15..=15).enumerate() {
        let n = u32::try_from(n).unwrap();
        tree.insert(ip(n, i as f32, i as f32), n);
    }

    let mut points = Vec::new();
    tree.find_range(Point::new(-10., -10.), r32(3.5), |p, v| points.push((p, v)));
    assert_eq!(
        points,
        vec![
            (ip(5, -10.0, -10.0), &5),
            (ip(6, -9.0, -9.0), &6),
            (ip(7, -8.0, -8.0), &7),
            (ip(3, -12.0, -12.0), &3),
            (ip(4, -11.0, -11.0), &4),
        ]
    );
}
