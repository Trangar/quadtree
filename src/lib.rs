//! Contains a [`QuadTree`] implementation that allows for efficient storage and lookup of values in a 2D space.
//!
//! This will be able to store up to `4 ^ 15 * N` items, about 1073 million times N.

#![warn(clippy::pedantic, missing_docs)]

mod bucket;
mod index;
mod point;
mod tests;

use bucket::Bucket;
use index::Index;
use point::Rect;
use smallvec::SmallVec;
use std::collections::BTreeMap;

pub use bucket::IdentityPoint;
pub use noisy_float::types::R32;
pub use point::Point;

/// The quad tree implementation. This is generic over value `T`, with bucket size of `N`. Each item should have unique identity `ID`
///
/// This tree will split when more than `n` items are inserted. Each split will have its own capacity of `N` items.
///
/// `N` should be a value of 1 or larger. A good value will depend on the size of `T` and how evenly distributed the data points are, and will only matter in how much memory is allocated.
///
/// A good starting value for `N` is 4.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuadTree<T, ID, const N: usize> {
    rect: point::Rect,
    items: Vec<Bucket<T, ID, N>>,
    outside_of_range: BTreeMap<ID, (T, Point)>,
    identity_to_point: BTreeMap<ID, (Point, Option<Index>)>,
}

impl<T, ID, const N: usize> QuadTree<T, ID, N>
where
    ID: std::cmp::Ord + std::fmt::Display + Clone + std::cmp::PartialEq<ID>,
{
    /// Create a new [`QuadTree`] which covers the area between `top_left` and `bottom_right`. Points outside of this range will be inserted in a slow [`BTreeMap`], so choose this value carefully.
    ///
    /// When dealing with a perfect rectangle around point `0, 0`, you can use `sized_around_origin` instead
    #[must_use]
    pub fn new(top_left: Point, bottom_right: Point) -> Self {
        Self {
            rect: point::Rect::new(top_left, bottom_right),
            items: vec![Bucket::Owned(SmallVec::new_const())],
            outside_of_range: BTreeMap::new(),
            identity_to_point: BTreeMap::new(),
        }
    }

    /// Create a new [`QuadTree`] which is centered around `0, 0`. and will span from `-size` to `size`.
    ///
    /// Points outside of this range will be inserted in a slow [`BTreeMap`], so choose this value carefully.
    #[must_use]
    pub fn sized_around_origin(size: Point) -> Self {
        Self::new(-size, size)
    }

    /// Insert a value `value` at the given `point`. If the existing `point.identity` already exists, it will be updated instead.
    pub fn insert(&mut self, point: IdentityPoint<ID>, value: T) {
        if let Some((_, old_index)) = self.identity_to_point.remove(&point.identity) {
            let mut value = Some(value);
            let new_index =
                self.update_inner(&point.identity, point.point, old_index, |old_value, idx| {
                    if let Some(value) = value.take() {
                        *old_value = value;
                    }
                    idx
                });
            self.identity_to_point
                .insert(point.identity, (point.point, new_index));
            return;
        }
        if !self.rect.contains(point.point) {
            self.outside_of_range
                .insert(point.identity.clone(), (value, point.point));
            self.identity_to_point
                .insert(point.identity, (point.point, None));
            return;
        }
        let index = Self::find_bucket_mut(
            &mut self.items,
            &mut self.identity_to_point,
            self.rect,
            point.point,
            true,
            |bucket, index| {
                bucket.push((point.clone(), value));
                index
            },
        );
        self.identity_to_point
            .insert(point.identity, (point.point, Some(index)));
    }

    /// Update the given identity to the new point.
    ///
    /// Will return `true` if the identity was found and updated, `false` otherwise
    pub fn update(&mut self, identity: ID, point: Point) -> bool {
        if let Some((_, maybe_index)) = self.identity_to_point.remove(&identity) {
            let new_idx = self.update_inner(&identity, point, maybe_index, |_, new_idx| new_idx);
            self.identity_to_point.insert(identity, (point, new_idx));
            true
        } else {
            false
        }
    }

    /// Update the given identity to the new point, with the opportunity to update the value
    ///
    /// Will return `true` if the identity was found and updated, `false` otherwise
    pub fn update_point_and_value(
        &mut self,
        identity: ID,
        point: Point,
        callback: impl FnOnce(&mut T),
    ) -> bool {
        if let Some((_, maybe_index)) = self.identity_to_point.remove(&identity) {
            let new_idx = self.update_inner(&identity, point, maybe_index, |val, new_idx| {
                callback(val);
                new_idx
            });
            self.identity_to_point.insert(identity, (point, new_idx));
            true
        } else {
            false
        }
    }

    /// Remove an entry with the given identity from the quad tree. For a non-panicing version use [`try_remove`]
    ///
    /// # Panics
    ///
    /// Will panic if the identity is not found in this quad tree.
    pub fn remove(&mut self, identity: &ID) -> (T, Point) {
        self.try_remove(identity)
            .unwrap_or_else(|| panic!("Identity {identity} not found"))
    }

    /// Try to remove the entry with the given identity from this quad tree. Will return the entry and the last know position if it's found, `None` otherwise.
    #[allow(clippy::missing_panics_doc)] // should not panic unless the internal state is wrong
    pub fn try_remove(&mut self, identity: &ID) -> Option<(T, Point)> {
        let (point, index) = self.identity_to_point.remove(identity)?;
        if let Some(index) = index {
            let result = self.items[index.to_idx()]
                .as_owned_mut()
                .remove_by_identity(identity);

            if let Some(parent) = index.parent() {
                self.try_merge(parent);
            }
            Some((result, point))
        } else {
            self.outside_of_range.remove(identity)
        }
    }

    /// Find all entries with a distance less than `range` away from point `center`. Each entry found will be passed to `callback`.
    ///
    /// `point` can be a point outside of this [`QuadTree`].
    pub fn find_range<'a>(
        &'a self,
        center: Point,
        range: R32,
        mut callback: impl FnMut(&ID, Point, &'a T),
    ) {
        let ctx = FindRangeCtx::new(center, range);

        self.find_range_inner(self.rect, Index::ROOT, &ctx, &mut callback);

        for (ip, (value, point)) in &self.outside_of_range {
            if ctx.point_in_range(*point) {
                callback(ip, *point, value);
            }
        }
    }
}

impl<T, ID, const N: usize> QuadTree<T, ID, N>
where
    ID: std::cmp::Ord + std::fmt::Display + Clone,
{
    fn update_inner<R>(
        &mut self,
        identity: &ID,
        new_point: Point,
        old_index: Option<Index>,
        callback: impl FnOnce(&mut T, Option<Index>) -> R,
    ) -> R {
        let mut callback = Some(callback);
        // get the new index first
        let new_index = if self.rect.contains(new_point) {
            let (result, idx) = Self::find_bucket_mut(
                &mut self.items,
                &mut self.identity_to_point,
                self.rect,
                new_point,
                false,
                |bucket, idx| {
                    // if the new index is the same as the old index, we just update it in-place and early return
                    if Some(idx) == old_index {
                        if let Some(n) = bucket.iter().position(|(ip, _)| &ip.identity == identity)
                        {
                            let (ip, t) = &mut bucket[n];
                            let result = (callback.take().unwrap())(t, Some(idx));
                            ip.point = new_point;
                            return (Some(result), idx);
                        }
                    }
                    (None, idx)
                },
            );
            if let Some(result) = result {
                return result;
            }
            Some(idx)
        } else {
            None
        };

        // We cannot update in-place, remove the old value and re-insert it
        let mut value = if let Some(idx) = old_index {
            self.items[idx.to_idx()]
                .as_owned_mut()
                .remove_by_identity(identity)
        } else {
            self.outside_of_range.remove(identity).unwrap().0
        };
        if let Some(index) = new_index {
            // new point is in this quad tree, quick insert it
            let bucket = self.items[index.to_idx()].as_owned_mut();
            let (smallvec, new_index) =
                if bucket.requires_split(self.rect.get_index_rect(index), Some(new_point)) {
                    let (new_vec, new_index) = Self::split(
                        &mut self.items,
                        &mut self.identity_to_point,
                        self.rect,
                        index,
                        new_point,
                    );
                    (new_vec, Some(new_index))
                } else {
                    (bucket.into_inner(), new_index)
                };
            let result = (callback.take().unwrap())(&mut value, new_index);
            smallvec.push((
                IdentityPoint::<ID> {
                    point: new_point,
                    identity: identity.clone(),
                },
                value,
            ));
            result
        } else {
            // new point is out of range of this quad tree, simply insert it into `out_of_range`
            let result = (callback.take().unwrap())(&mut value, None);
            self.outside_of_range
                .insert(identity.clone(), (value, new_point));
            result
        }
    }

    fn find_bucket_mut<R>(
        items: &mut Vec<Bucket<T, ID, N>>,
        identity_to_point: &mut BTreeMap<ID, (Point, Option<Index>)>,
        mut rect: point::Rect,
        point: Point,
        require_resize: bool,
        cb: impl FnOnce(&mut SmallVec<[(IdentityPoint<ID>, T); N]>, Index) -> R,
    ) -> R {
        let mut index = Index::ROOT;
        loop {
            let bucket = ensure_index_valid(items, index);
            match bucket {
                Bucket::Nested => {
                    let (new_rect, quadrant) = rect.get_quadrant(point);
                    index = index.child_at(quadrant);
                    rect = new_rect;
                }
                Bucket::Owned(smallvec) => {
                    let (smallvec, index) = if require_resize {
                        if smallvec.len() < N {
                            return cb(smallvec, index);
                        }

                        Self::split(items, identity_to_point, rect, index, point)
                    } else {
                        (smallvec, index)
                    };
                    return cb(smallvec, index);
                }
            }
        }
    }

    fn split<'a>(
        items: &'a mut Vec<Bucket<T, ID, N>>,
        identity_to_point: &mut BTreeMap<ID, (Point, Option<Index>)>,
        rect: point::Rect,
        index: Index,
        point: Point,
    ) -> (&'a mut SmallVec<[(IdentityPoint<ID>, T); N]>, Index) {
        let new_item_quadrant = rect.get_quadrant(point).1;

        if let Some(Bucket::Owned(smallvec)) = items.get_mut(index.to_idx()) {
            if smallvec
                .iter()
                .all(|(ip, _)| rect.get_quadrant(ip.point).1 == new_item_quadrant)
            {
                // special case: all of these positions are on the same quadrant, so we cannot split
                // therefor we must overflow the smallvec

                // safety: rust lifetimes are jank and the compiler thinks this is still borrowed below even though we clearly return
                // so this breaks that issue
                return (unsafe { &mut *(smallvec as *mut _) }, index);
            }
        } else {
            panic!(
                "Item at index {index:?} ({}) is nested, but we tried to split it",
                index.to_idx()
            );
        }
        // debug_tree(items, Index::ROOT);

        // we should be able to safely split
        ensure_index_valid(items, index.child_at(new_item_quadrant));
        let Bucket::Owned(smallvec) = std::mem::replace(&mut items[index.to_idx()], Bucket::Nested) else { unreachable!() };

        for (point, value) in smallvec {
            let (mut rect, quadrant) = rect.get_quadrant(point.point);
            let mut index = index.child_at(quadrant);
            let (smallvec, index) = loop {
                ensure_index_valid(items, index);
                match &mut items[index.to_idx()] {
                    Bucket::Owned(smallvec) => break (smallvec, index),
                    Bucket::Nested => {
                        let (new_rect, quadrant) = rect.get_quadrant(point.point);
                        rect = new_rect;
                        index = index.child_at(quadrant);
                    }
                }
            };
            smallvec.push((point.clone(), value));

            identity_to_point.insert(point.identity, (point.point, Some(index)));
        }
        let mut index = index.child_at(new_item_quadrant);
        let mut rect = rect;
        loop {
            match ensure_index_valid(items, index) {
                Bucket::Owned(smallvec) => {
                    // rust lifetimes again, we're `break`ing here but it keeps the lifetime for the next iteration
                    let smallvec = unsafe { &mut *(smallvec as *mut _) };
                    break (smallvec, index);
                }
                Bucket::Nested => {
                    let (new_rect, quadrant) = rect.get_quadrant(point);
                    rect = new_rect;
                    index = index.child_at(quadrant);
                }
            };
        }
    }

    fn try_merge(&mut self, index: Index) {
        let Some(children) = index.children() else { return };
        let sum = children.iter().fold(0, |acc, idx| {
            acc + if let Bucket::Owned(n) = &self.items[idx.to_idx()] {
                n.len()
            } else {
                N + 1
            }
        });
        if sum <= N {
            let mut parent = SmallVec::new();
            for child_idx in children {
                let Bucket::Owned(n) = std::mem::replace(&mut self.items[child_idx.to_idx()], Bucket::Nested) else { unreachable!() };
                for (ip, value) in n {
                    self.identity_to_point
                        .insert(ip.identity.clone(), (ip.point, Some(index)));
                    parent.push((ip, value));
                }
            }
            debug_assert!(matches!(self.items[index.to_idx()], Bucket::Nested));
            self.items[index.to_idx()] = Bucket::Owned(parent);

            while self.items.len() > 1 && matches!(self.items.last(), Some(Bucket::Nested)) {
                self.items.pop();
            }
        }
    }

    fn find_range_inner<'a>(
        &'a self,
        rect: Rect,
        index: Index,
        ctx: &FindRangeCtx,
        callback: &mut impl FnMut(&ID, Point, &'a T),
    ) {
        if !ctx.contains_rect(rect) {
            return;
        }
        match self.items.get(index.to_idx()) {
            Some(Bucket::Owned(items)) => {
                for (ident, val) in items {
                    if ctx.point_in_range(ident.point) {
                        callback(&ident.identity, ident.point, val);
                    }
                }
            }
            Some(Bucket::Nested) => {
                for child in point::Quadrant::all() {
                    let rect = rect.get_child_at(child);
                    let index = index.child_at(child);
                    self.find_range_inner(rect, index, ctx, callback);
                }
            }
            None => {}
        }
    }
}

struct FindRangeCtx {
    center: Point,
    range_squared: R32,
    full_rect: Rect,
}
impl FindRangeCtx {
    fn new(center: Point, range: R32) -> Self {
        Self {
            center,
            range_squared: range * range,
            full_rect: Rect::new(center - range, center + range),
        }
    }
    fn contains_rect(&self, rect: Rect) -> bool {
        self.full_rect.intersects(rect)
    }

    fn point_in_range(&self, point: Point) -> bool {
        self.center.distance_squared_to(point) <= self.range_squared
    }
}

fn ensure_index_valid<T, ID, const N: usize>(
    items: &mut Vec<Bucket<T, ID, N>>,
    index: Index,
) -> &mut Bucket<T, ID, N> {
    if let Some(index) = index.parent() {
        let index = index.child_at(point::Quadrant::BottomRight);
        let new_max_item_count = index.to_idx();
        if items.len() <= new_max_item_count {
            items.resize_with(new_max_item_count + 1, || Bucket::Owned(SmallVec::new()));
        }
    } else {
        debug_assert!(!items.is_empty());
    };

    // Safety: We just asserted that this vec has this index
    unsafe { items.get_unchecked_mut(index.to_idx()) }
}

// fn debug_tree<T: std::fmt::Debug, const N: usize>(items: &[Bucket<T, ID, N>], index: Index) {
//     let ident = String::from(' ').repeat(index.depth());
//     match items.get(index.to_idx()) {
//         Some(Bucket::Nested) => {
//             println!("{ident}Nested ({index:?})");
//             for child in index.children().unwrap() {
//                 debug_tree(items, child);
//             }
//         }
//         Some(Bucket::Owned(n)) => {
//             println!("{ident}Owned ({index:?}, {} items)", n.len());
//             for (ip, t) in n {
//                 println!("{ident}- {ip:?} - {t:?}");
//             }
//         }
//         None => {}
//     }
// }
