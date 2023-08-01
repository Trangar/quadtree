//! This fuzzer runs 2 systems in parallel, the quad tree and a simple flat map.
//!
//! It builds up a list of [`Instruction`] based on the input bytes. It then runs those instructions on both the tree and the flat map, and checks if they generate the same output.

#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use whquadtree::{IdentityPoint, Point, QuadTree, R32};

fuzz_target!(|instructions: Vec<Instruction>| {
    let mut tree = QuadTree::sized_around_origin(Point::new(10., 10.));
    let mut flat = Vec::new();
    // println!("Instructions:");
    // for instruction in &instructions {
    //     println!(" - {instruction:?}");
    // }
    for instruction in instructions.clone() {
        instruction.execute(&mut tree, &mut flat, &instructions);
    }
});

#[derive(Debug, Clone, Arbitrary)]
enum Instruction {
    Insert {
        identity: u32,
        #[arbitrary(with = arbitrary_r32)]
        x: R32,
        #[arbitrary(with = arbitrary_r32)]
        y: R32,
        value: u32,
    },
    Update {
        identity: u32,
        #[arbitrary(with = arbitrary_r32)]
        x: R32,
        #[arbitrary(with = arbitrary_r32)]
        y: R32,
    },
    UpdateValue {
        identity: u32,
        #[arbitrary(with = arbitrary_r32)]
        x: R32,
        #[arbitrary(with = arbitrary_r32)]
        y: R32,
        value: u32,
    },
    Remove {
        identity: u32,
    },
    FindRange {
        #[arbitrary(with = arbitrary_r32)]
        x: R32,
        #[arbitrary(with = arbitrary_r32)]
        y: R32,
        #[arbitrary(with = arbitrary_r32)]
        range: R32,
    },
}

fn arbitrary_r32(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<R32> {
    R32::try_new(u.arbitrary()?).ok_or(arbitrary::Error::IncorrectFormat)
}

impl Instruction {
    fn execute(
        self,
        tree: &mut QuadTree<u32, u32, 4>,
        flat: &mut Vec<(u32, Point, u32)>,
        instructions: &[Instruction],
    ) {
        match self {
            Self::Insert {
                identity,
                x,
                y,
                value,
            } => {
                let ident = IdentityPoint {
                    identity,
                    point: Point::new_noisy_float(x, y),
                };
                tree.insert(ident, value);
                flat.retain(|(i, _, _)| *i != identity);
                flat.push((ident.identity, ident.point, value));
            }
            Self::Update { identity, x, y } => {
                tree.update(identity, Point::new_noisy_float(x, y));
                for (i, point, _) in flat.iter_mut() {
                    if *i == identity {
                        *point = Point::new_noisy_float(x, y);
                        break;
                    }
                }
            }
            Self::UpdateValue {
                identity,
                x,
                y,
                value,
            } => {
                tree.update_point_and_value(identity, Point::new_noisy_float(x, y), |v| *v = value);
                for (ident, point, v) in flat.iter_mut() {
                    if *ident == identity {
                        *point = Point::new_noisy_float(x, y);
                        *v = value;
                        break;
                    }
                }
            }
            Self::Remove { identity } => {
                let flat_idx = flat.iter().position(|(id, _, _)| *id == identity);
                if let Some((value, point)) = tree.try_remove(&identity) {
                    let (_, flat_point, flat_value) = flat.remove(flat_idx.unwrap());
                    assert_eq!(value, flat_value);
                    assert_eq!(point, flat_point);
                } else {
                    assert!(flat_idx.is_none());
                }
            }
            Self::FindRange { x, y, range } => {
                if R32::try_new(x.raw() - range.raw()).is_none()
                    || R32::try_new(x.raw() + range.raw()).is_none()
                    || R32::try_new(y.raw() - range.raw()).is_none()
                    || R32::try_new(y.raw() + range.raw()).is_none()
                    || R32::try_new(range.raw() * range.raw()).is_none()
                {
                    return;
                }

                let mut tree_items = Vec::new();
                let center = Point::new_noisy_float(x, y);
                let range_squared = range * range;
                tree.find_range(center, range, |id, point, value| {
                    tree_items.push((*id, point, *value))
                });

                let mut flat_items = flat
                    .iter()
                    .filter(|(_, point, _)| center.distance_squared_to(*point) <= range_squared)
                    .copied()
                    .collect::<Vec<_>>();

                tree_items.sort_by_key(|(identity, _, _)| *identity);
                flat_items.sort_by_key(|(identity, _, _)| *identity);

                if tree_items != flat_items {
                    println!("Tree items do not match flat items");

                    println!("Quad tree: {tree:?}");
                    println!("Flat tree: {flat:?}");

                    println!("Tree items: {tree_items:?}");
                    println!("Flat items: {flat_items:?}");

                    println!("Instructions:");
                    for instruction in instructions {
                        println!(" - {instruction:?}");
                    }
                    panic!();
                }
            }
        }
    }
}
