//! # Internal documentation:
//!
//! We're using a flat buffer to store all the items. The root node is located at 0b1.
//! For each split, we shift this value to the left by 2, and use the new 2 bits to encode what quadrant it is
//!
//! |binary|location      |
//! |------|--------------|
//! | 0b00 | top-left     |
//! | 0b01 | top-right    |
//! | 0b10 | bottom-left  |
//! | 0b11 | bottom-right |
//!
//! So depth = 1, bottom-right would be `(0b1 << 2) | 0b11` -> `0b111` -> 5
// Use a `u32` internally, which allows for 15 layers, allowing for 4^15 * N entries to be stored

use crate::point::Quadrant;
use std::{fmt, num::NonZeroU32};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index(NonZeroU32);

impl fmt::Debug for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Index").field(&PrintIndex(self)).finish()
    }
}

struct PrintIndex<'a>(&'a Index);

impl fmt::Debug for PrintIndex<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "1")?;
        for quadrant in self.0.iter_from_root() {
            write!(f, "_{:02b}", quadrant as u8)?;
        }
        Ok(())
    }
}

impl Index {
    pub const ROOT: Index = Index(unsafe { NonZeroU32::new_unchecked(1) });

    ///
    /// # Panics
    ///
    /// Will panic if given too many quadrants. This only supports between 0 and 15 (inclusive) items.
    #[cfg(test)]
    pub fn from_quadrants(quadrants: impl Iterator<Item = Quadrant>) -> Self {
        let mut n = 1;
        for (idx, quad) in quadrants.enumerate() {
            assert!(idx <= 15, "Given quadrant iterator is too deep");
            n = (n << 2) | (quad as u32);
        }
        Self(NonZeroU32::new(n).unwrap())
    }

    // /// Get the depth of this index. The depth of the root node is 0, and each layer below that is 1 deeper.
    // ///
    // /// To get the depth of the current index, we get the total bit count, subtract the leading zeroes, -1 / 2, and we get the depth
    // /// ```no_compile
    // /// 0b1 -> 31 (or 63)  -> 1 -> 1-1/2 -> 0
    // /// 0b111 -> 28 (or 60)  -> 3 -> 3-1/2 -> 1
    // /// ```
    // pub const fn depth(self) -> usize {
    //     let leading_zeroes = self.0.leading_zeros();
    //     let total_bits = NonZeroU32::BITS;

    //     (((total_bits - leading_zeroes) - 1) / 2) as usize
    // }

    pub const fn iter_from_root(self) -> FromRootIterator {
        FromRootIterator {
            idx: self.0.leading_zeros() + 1,
            n: self.0.get(),
        }
    }

    #[cfg(test)]
    pub const fn iter_from_leaf(self) -> FromLeafIterator {
        FromLeafIterator { n: self.0.get() }
    }

    // /// Returns `true` if this index is the root node.
    // pub const fn is_root(self) -> bool {
    //     self.0.get() == 0b1
    // }

    /// Get the parent of this index.
    ///
    /// This will fail if this index is the root node.
    pub const fn parent(self) -> Option<Index> {
        if let Some(index) = NonZeroU32::new(self.0.get() >> 2) {
            Some(Self(index))
        } else {
            None
        }
    }

    /// Get the children indexes of this index
    #[allow(clippy::unusual_byte_groupings, clippy::identity_op)] // clippy is wrong
    pub const fn children(self) -> Option<[Index; 4]> {
        // make sure that `N` did not overflow. This can happen if we are at depth 15 and try to shift.
        // At depth 15 this index is `0b01xxxxxx_xxxxxxxx_xxxxxxxx_xxxxxxxx`
        // so we check for `n >= 0b01000000_00000000_0000000_000000000`
        if self.0.get() >= 0b01000000_00000000_0000000_000000000 {
            None
        } else {
            let n = self.0.get() << 2;
            // These `NonZeroU32`s can only fail if `n` overflowed, but we asserted above that we don't overflow
            Some(unsafe {
                [
                    Index(NonZeroU32::new_unchecked(n | 0b00)),
                    Index(NonZeroU32::new_unchecked(n | 0b01)),
                    Index(NonZeroU32::new_unchecked(n | 0b10)),
                    Index(NonZeroU32::new_unchecked(n | 0b11)),
                ]
            })
        }
    }

    pub const fn to_idx(self) -> usize {
        const MASK: u32 = 0xAAAA_AAAA;
        let offset = MASK & (u32::MAX >> self.0.leading_zeros());
        (self.0.get() - offset - 1) as usize
    }

    pub(crate) fn child_at(self, quadrant: crate::point::Quadrant) -> Index {
        Index(NonZeroU32::new((self.0.get() << 2) | quadrant as u32).unwrap())
    }
}

pub struct FromRootIterator {
    idx: u32,
    n: u32,
}

impl Iterator for FromRootIterator {
    type Item = Quadrant;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < 32 {
            let bytes = (self.n >> (30 - self.idx)) & 0b11;
            self.idx += 2;
            Some(Quadrant::from_bits(bytes as u8))
        } else {
            None
        }
    }
}

impl ExactSizeIterator for FromRootIterator {
    fn len(&self) -> usize {
        ((32 - self.idx) / 2) as usize
    }
}

impl DoubleEndedIterator for FromRootIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.idx < 32 {
            let bytes = self.n & 0b11;
            self.idx += 1;
            self.n >>= 2;
            Some(Quadrant::from_bits(bytes as u8))
        } else {
            None
        }
    }
}

#[cfg(test)]
pub struct FromLeafIterator {
    n: u32,
}

#[cfg(test)]
impl Iterator for FromLeafIterator {
    type Item = Quadrant;

    fn next(&mut self) -> Option<Self::Item> {
        if self.n == 1 {
            None
        } else {
            let bytes = self.n & 0b11;
            self.n >>= 2;
            Some(Quadrant::from_bits(bytes as u8))
        }
    }
}

#[cfg(test)]
impl ExactSizeIterator for FromLeafIterator {
    fn len(&self) -> usize {
        ((32 - (self.n.leading_zeros() + 1)) / 2) as usize
    }
}

#[cfg(test)]
fn permutations(total_len: usize, mut permutation: impl FnMut(Index, &[Quadrant])) {
    for i in 0..=total_len {
        let mut quadrants = vec![Quadrant::from_bits(0b00); i];
        loop {
            let index = Index::from_quadrants(quadrants.iter().copied());
            permutation(index, &quadrants);
            let Some(last_idx_not_0b11) = quadrants.iter().rposition(|q| (*q as u8) != 0b11) else { break };

            if last_idx_not_0b11 != quadrants.len() - 1 {
                for quad in &mut quadrants[last_idx_not_0b11 + 1..] {
                    *quad = Quadrant::from_bits(0b00);
                }
            }
            quadrants[last_idx_not_0b11] =
                Quadrant::from_bits(quadrants[last_idx_not_0b11] as u8 + 1);
        }
    }
}
#[test]
fn validate_quadrants() {
    let total_len = if cfg!(feature = "slow-tests") { 15 } else { 8 };
    permutations(total_len, |index, quadrants| {
        let reformed = index.iter_from_root().collect::<Vec<_>>();
        assert_eq!(quadrants, reformed);

        let reversed = index.iter_from_leaf().collect::<Vec<_>>();
        let mut quadrants_reversed = quadrants.to_vec();
        quadrants_reversed.reverse();
        assert_eq!(quadrants_reversed, reversed);
    });
}

#[test]
fn validate_index() {
    let total_len: u32 = if cfg!(feature = "slow-tests") { 15 } else { 10 };
    let map_length = (0..=total_len).fold(0, |acc, add| acc + 4u32.pow(add));
    let mut map = vec![Option::<Index>::None; map_length as usize];
    permutations(total_len as usize, |index, _| {
        let prev = &mut map[index.to_idx()];
        assert!(
            !prev.is_some(),
            "Tried to set {index:?} at index {} but {:?} was already set",
            index.to_idx(),
            prev.unwrap()
        );
        *prev = Some(index);
    });
    assert!(
        map.iter().all(Option::is_some),
        "{map:?} is not completely filled"
    );
}
