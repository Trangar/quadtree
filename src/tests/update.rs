#![allow(clippy::cast_precision_loss, clippy::too_many_lines)]
#![cfg(test)]

use std::collections::BTreeMap;

use crate::{
    bucket::Bucket,
    index::Index,
    tests::{ip, ipv},
    Point, QuadTree,
};
use smallvec::smallvec;

#[test]
fn validate_simple() {
    let mut tree = QuadTree::<u32, u32, 4>::sized_around_origin(Point::new(10., 10.));
    tree.insert(ip(1, 1.0, 1.0), 1);

    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![ipv(1, 1.0, 1.0, 1)])]
    );
    for n in 2..5 {
        tree.insert(ip(n, n as f32, n as f32), n);
    }
    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![
            ipv(1, 1.0, 1.0, 1),
            ipv(2, 2.0, 2.0, 2),
            ipv(3, 3.0, 3.0, 3),
            ipv(4, 4.0, 4.0, 4),
        ])]
    );

    assert!(tree.update(1, Point::new(2.5, 2.5)));

    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![
            ipv(1, 2.5, 2.5, 1),
            ipv(2, 2.0, 2.0, 2),
            ipv(3, 3.0, 3.0, 3),
            ipv(4, 4.0, 4.0, 4),
        ])]
    );
}

#[test]
fn validate_nested() {
    let mut tree = QuadTree::<i32, u32, 4>::sized_around_origin(Point::new(10., 10.));
    for n in -2..=2 {
        tree.insert(ip(u32::try_from(n + 2).unwrap(), n as f32, n as f32), n);
    }
    assert_eq!(
        tree.items,
        vec![
            Bucket::Nested,
            Bucket::Owned(smallvec![ipv(0, -2., -2., -2), ipv(1, -1., -1., -1)]),
            Bucket::Owned(smallvec![]),
            Bucket::Owned(smallvec![]),
            Bucket::Owned(smallvec![
                ipv(2, 0.0, 0.0, 0),
                ipv(3, 1.0, 1.0, 1),
                ipv(4, 2.0, 2.0, 2),
            ]),
        ]
    );

    assert!(tree.update(1, Point::new(-1., -1.)));

    assert_eq!(
        tree.items,
        vec![
            Bucket::Nested,
            Bucket::Owned(smallvec![ipv(0, -2., -2., -2), ipv(1, -1., -1., -1)]),
            Bucket::Owned(smallvec![]),
            Bucket::Owned(smallvec![]),
            Bucket::Owned(smallvec![
                ipv(2, 0.0, 0.0, 0),
                ipv(3, 1.0, 1.0, 1),
                ipv(4, 2.0, 2.0, 2),
            ]),
        ]
    );
}

#[test]
fn validate_out_of_range() {
    let mut tree = QuadTree::<u32, u32, 4>::sized_around_origin(Point::new(10., 10.));
    tree.insert(ip(1, 1.0, 1.0), 1);
    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![ipv(1, 1.0, 1.0, 1)])]
    );
    assert_eq!(tree.outside_of_range, BTreeMap::default());
    tree.insert(ip(1, 11.0, 11.0), 1);
    assert_eq!(tree.items, vec![Bucket::Owned(smallvec![])]);
    assert_eq!(
        tree.outside_of_range,
        [(1, (1, Point::new(11., 11.)))].into_iter().collect()
    );
    tree.insert(ip(1, 1.0, 1.0), 1);
    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![ipv(1, 1.0, 1.0, 1)])]
    );
    assert_eq!(tree.outside_of_range, BTreeMap::default());
}

#[test]
fn validate_update() {
    let mut tree = QuadTree::<u32, u32, 4>::sized_around_origin(Point::new(10., 10.));
    tree.insert(ip(1, 1.0, 1.0), 1);
    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![ipv(1, 1.0, 1.0, 1)])]
    );
    assert_eq!(tree.outside_of_range, BTreeMap::default());
    assert_eq!(
        tree.identity_to_point,
        [(1, (Point::new(1., 1.), Some(Index::ROOT)))]
            .into_iter()
            .collect()
    );

    assert!(tree.update_point_and_value(1, Point::new(1.0, 1.0), |v| *v += 1));
    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![ipv(1, 1.0, 1.0, 2)])]
    );
    assert_eq!(tree.outside_of_range, BTreeMap::default());
    assert_eq!(
        tree.identity_to_point,
        [(1, (Point::new(1., 1.), Some(Index::ROOT)))]
            .into_iter()
            .collect()
    );

    assert!(tree.update_point_and_value(1, Point::new(11.0, 11.0), |v| *v += 1));
    assert_eq!(tree.items, vec![Bucket::Owned(smallvec![])]);
    assert_eq!(
        tree.outside_of_range,
        [(1, (3, Point::new(11., 11.)))].into_iter().collect()
    );
    assert_eq!(
        tree.identity_to_point,
        [(1, (Point::new(11., 11.), None))].into_iter().collect()
    );

    assert!(tree.update_point_and_value(1, Point::new(1.0, 1.0), |v| *v += 1));
    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![ipv(1, 1.0, 1.0, 4)])]
    );
    assert_eq!(tree.outside_of_range, BTreeMap::default());
    assert_eq!(
        tree.identity_to_point,
        [(1, (Point::new(1., 1.), Some(Index::ROOT)))]
            .into_iter()
            .collect()
    );
}
