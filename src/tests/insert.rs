#![allow(clippy::cast_precision_loss, clippy::too_many_lines)]
#![cfg(test)]

use crate::{
    bucket::Bucket,
    tests::{ip, ipv},
    Point, QuadTree,
};
use smallvec::{smallvec, SmallVec};

#[test]
fn validate() {
    let mut tree = QuadTree::<u32, u32, 4>::new(Point::zero(), Point::new(8., 8.));
    tree.insert(ip(1, 1.0, 1.0), 1);

    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![ipv(1, 1.0, 1.0, 1)])],
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
        ]),]
    );
    tree.insert(ip(5, 5.0, 5.0), 5);
    assert_eq!(
        tree.items,
        vec![
            Bucket::Nested,
            Bucket::Owned(smallvec![
                ipv(1, 1.0, 1.0, 1),
                ipv(2, 2.0, 2.0, 2),
                ipv(3, 3.0, 3.0, 3),
            ]),
            Bucket::Owned(SmallVec::new()),
            Bucket::Owned(SmallVec::new()),
            Bucket::Owned(smallvec![ipv(4, 4.0, 4.0, 4), ipv(5, 5.0, 5.0, 5),])
        ]
    );

    tree.insert(ip(6, 6.0, 6.0), 6);
    assert_eq!(
        tree.items,
        vec![
            Bucket::Nested,
            Bucket::Owned(smallvec![
                ipv(1, 1.0, 1.0, 1),
                ipv(2, 2.0, 2.0, 2),
                ipv(3, 3.0, 3.0, 3),
            ]),
            Bucket::Owned(SmallVec::new()),
            Bucket::Owned(SmallVec::new()),
            Bucket::Owned(smallvec![
                ipv(4, 4.0, 4.0, 4),
                ipv(5, 5.0, 5.0, 5),
                ipv(6, 6.0, 6.0, 6),
            ])
        ]
    );
}

#[test]
fn out_of_bounds() {
    let mut tree = QuadTree::<u32, u32, 4>::sized_around_origin(Point::new(20., 20.));
    tree.insert(ip(0, 25., 25.), 0);
    assert_eq!(tree.items, vec![Bucket::Owned(smallvec![])]);
    assert_eq!(tree.outside_of_range.len(), 1);
    assert_eq!(tree.identity_to_point.len(), 1);
    tree.insert(ip(1, 15., 15.), 1);
    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![ipv(1, 15., 15., 1)])]
    );
    assert_eq!(tree.outside_of_range.len(), 1);
    assert_eq!(tree.identity_to_point.len(), 2);

    assert!(tree.update(0, Point::new(20., 20.)));
    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![
            ipv(1, 15., 15., 1),
            ipv(0, 20., 20., 0)
        ])]
    );
    assert_eq!(tree.outside_of_range.len(), 0);
    assert_eq!(tree.identity_to_point.len(), 2);

    assert!(tree.update(0, Point::new(25., 25.)));
    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![ipv(1, 15., 15., 1),])]
    );
    assert_eq!(tree.outside_of_range.len(), 1);
    assert_eq!(tree.identity_to_point.len(), 2);
}

#[test]
fn insert_same_identity() {
    let mut tree = QuadTree::<u32, u32, 4>::sized_around_origin(Point::new(20., 20.));
    assert_eq!(tree.items, vec![Bucket::Owned(SmallVec::new())]);
    tree.insert(ip(1, 0.0, 0.0), 1);
    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![ipv(1, 0.0, 0.0, 1)])]
    );
    tree.insert(ip(1, 1.0, 1.0), 1);
    assert_eq!(
        tree.items,
        vec![Bucket::Owned(smallvec![ipv(1, 1.0, 1.0, 1)])]
    );
}

#[allow(clippy::unreadable_literal)] // these values come from fuzzing, they're expected to be unreadable
#[test]
fn fuzz_results() {
    let mut tree = QuadTree::<u32, u32, 4>::sized_around_origin(Point::new(10., 10.));
    for (identity, x, y, value) in [
        (66681u32, 3.57e-43, 0.0, 2717566207u32),
        (11599872, 3.57e-43, 0.0, 0),
        (0, 5.96875, 2.75774e-40, 0),
        (3875537151, 0.0, -1.6992803e-18, 4294967295),
        (4281008127, 2.000267, 3.229e-42, 0),
        (50529027, 2.3509895e-38, 1.5045868e-36, 50529027),
    ] {
        tree.insert(
            crate::IdentityPoint {
                identity,
                point: Point::new(x, y),
            },
            value,
        );
        println!("---");
        println!("items: {:?}", tree.items);
        println!("itp: {:?}", tree.identity_to_point);
    }
    for identity in [50529027, 0] {
        let removed = tree.remove(&identity);
        println!("---");
        println!("Removed {removed:?}");
        println!("items: {:?}", tree.items);
        println!("itp: {:?}", tree.identity_to_point);
    }
    tree.insert(
        crate::IdentityPoint {
            identity: 0,
            point: Point::new(8.615331e-14, 9.404816e-38),
        },
        1086466304,
    );
}
