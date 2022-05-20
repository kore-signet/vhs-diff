use crate::*;
use std::ops::Range;

pub struct RkyvPatchesToVec<'a, T: 'a> {
    inner: &'a mut Vec<T>,
    seed: T,
    range: Range<usize>,
}

impl<'a, T: Patch + Clone> RkyvPatchesToVec<'a, T> {
    #[inline(always)]
    pub fn new(seed: T, inner: &'a mut Vec<T>) -> RkyvPatchesToVec<T> {
        RkyvPatchesToVec {
            inner,
            seed,
            range: usize::MIN..usize::MAX,
        }
    }

    #[inline(always)]
    pub fn new_range(seed: T, inner: &'a mut Vec<T>, range: Range<usize>) -> RkyvPatchesToVec<T> {
        RkyvPatchesToVec { inner, seed, range }
    }

    #[inline(always)]
    pub fn apply_range<'de, D>(
        seed: T,
        inner: &'a mut Vec<T>,
        range: Range<usize>,
        patches: &[ArchivedArchivablePatch],
    ) {
        (RkyvPatchesToVec { inner, seed, range }).apply(patches)
    }

    #[inline(always)]
    pub fn apply(self, patches: &[ArchivedArchivablePatch]) {
        let mut cur = self.seed;
        let mut i = 0;

        for patch in patches {
            unsafe { apply_rkyv_patch(&mut cur, patch) };

            if i >= self.range.end {
                break;
            }

            if self.range.contains(&i) {
                self.inner.push(cur.clone());
            }

            i += 1;
        }
    }
}

// applies a sequence of patches to an object, up to an index
pub struct ApplyRkyvPatches<'a, T: 'a>(&'a mut T, usize);

impl<'a, T: Patch> ApplyRkyvPatches<'a, T> {
    pub fn new(val: &'a mut T, limit: usize) -> Self {
        ApplyRkyvPatches(val, limit)
    }

    pub fn apply(val: &'a mut T, limit: usize, patches: &[ArchivedArchivablePatch]) {
        ApplyRkyvPatches(val, limit).run(patches)
    }

    pub fn run(self, patches: &[ArchivedArchivablePatch]) {
        let mut i = 0;

        for patch in patches {
            unsafe { apply_rkyv_patch(self.0, patch) };
            if i >= self.1 {
                break;
            }

            i += 1;
        }
    }
}
