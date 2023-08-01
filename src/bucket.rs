use crate::Point;
use smallvec::SmallVec;

/// A bucket that stores information on this bucket
///
/// If this is `Nested`, this bucket does not contain information, but the nested [`Index`]es should be scanned.
///
/// [`Index`]: ../index/struct.Index.html
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Bucket<T, const N: usize> {
    Nested = 1,
    Owned(SmallVec<[(IdentityPoint, T); N]>) = 2,
}
impl<T, const N: usize> Bucket<T, N> {
    pub(crate) fn as_owned_mut(&mut self) -> OwnedMut<T, N> {
        if let Self::Owned(v) = self {
            OwnedMut(v)
        } else {
            panic!("Bucket is not owned")
        }
    }
}

#[derive(Debug)]
pub struct OwnedMut<'a, T, const N: usize>(&'a mut SmallVec<[(IdentityPoint, T); N]>);

impl<'a, T, const N: usize> OwnedMut<'a, T, N> {
    pub(crate) fn remove_by_identity(&mut self, identity: u32) -> T {
        let idx = self
            .0
            .iter()
            .position(|(p, _)| p.identity == identity)
            .unwrap();
        self.0.remove(idx).1
    }

    // pub(crate) fn push(&mut self, ident: IdentityPoint, value: T) {
    //     self.0.push((ident, value));
    // }

    pub(crate) fn requires_split(
        &self,
        rect: super::point::Rect,
        point_to_add: Option<Point>,
    ) -> bool {
        if self.0.len() < N {
            false
        } else if let Some((first, remaining)) = self.0.split_first() {
            let quadrant = rect.get_quadrant(first.0.point);
            !remaining
                .iter()
                .map(|(ip, _)| ip.point)
                .chain(point_to_add)
                .all(|point| rect.get_quadrant(point) == quadrant)
        } else {
            false
        }
    }

    pub(crate) fn into_inner(self) -> &'a mut SmallVec<[(IdentityPoint, T); N]> {
        self.0
    }
}

#[test]
fn assert_smallvec_size() {
    // make sure we don't accidentally increase the size of a bucket by a change somewhere
    assert_eq!(
        std::mem::size_of::<Bucket<u32, 10>>(),
        std::mem::size_of::<Option<SmallVec<[(IdentityPoint, u32); 10]>>>()
    );
}

/// A [`Point`] an an identity. This is used to store and look up entries in the [`QuadTree`]
///
/// The quad tree will assume that entries with the same [`identity`] can be safely overwritten.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdentityPoint {
    ///
    pub identity: u32,
    ///
    // TODO: Can we get rid of this?
    pub point: Point,
}
