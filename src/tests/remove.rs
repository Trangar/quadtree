#![allow(clippy::cast_precision_loss)]

use crate::{
    bucket::Bucket,
    index::Index,
    tests::{ip, ipv},
    Point, QuadTree,
};
use smallvec::smallvec;

#[test]
fn in_range() {
    let mut tree = QuadTree::<u32, 4>::sized_around_origin(Point::new(10., 10.));

    for i in 0..5 {
        tree.insert(ip(i, i as f32 - 2.0, i as f32 - 2.0), i);
    }
    assert_eq!(
        tree.items,
        vec![
            Bucket::Nested,
            Bucket::Owned(smallvec![ipv(0, -2., -2., 0), ipv(1, -1., -1., 1),]),
            Bucket::Owned(smallvec![]),
            Bucket::Owned(smallvec![]),
            Bucket::Owned(smallvec![
                ipv(2, 0., 0., 2),
                ipv(3, 1.0, 1.0, 3),
                ipv(4, 2.0, 2.0, 4)
            ])
        ]
    );
    let (val, point) = tree.remove(0);
    assert_eq!(Point::new(-2., -2.), point);
    assert_eq!(0, val);
    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![
            ipv(1, -1., -1., 1),
            ipv(2, 0., 0., 2),
            ipv(3, 1.0, 1.0, 3),
            ipv(4, 2.0, 2.0, 4)
        ])]
    );
}

#[test]
fn out_of_range() {
    let mut tree = QuadTree::<u32, 4>::sized_around_origin(Point::new(10., 10.));
    for i in 0..5 {
        tree.insert(ip(i, i as f32 + 8.0, i as f32 + 8.0), i);
    }
    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![
            ipv(0, 8., 8., 0),
            ipv(1, 9., 9., 1),
            ipv(2, 10., 10., 2)
        ])]
    );
    assert_eq!(
        tree.outside_of_range,
        [ipv(3, 11., 11., 3), ipv(4, 12., 12., 4)]
            .into_iter()
            .collect()
    );
    tree.remove(3);
    assert_eq!(
        tree.outside_of_range,
        [ipv(4, 12., 12., 4)].into_iter().collect()
    );
    assert_eq!(
        tree.identity_to_point,
        [
            (0, (Point::new(8., 8.), Some(Index::ROOT))),
            (1, (Point::new(9., 9.), Some(Index::ROOT))),
            (2, (Point::new(10., 10.), Some(Index::ROOT))),
            (4, (Point::new(12., 12.), None))
        ]
        .into_iter()
        .collect()
    );
}
