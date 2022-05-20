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
    pub fn apply_range<'de, D>(seed: T, inner: &'a mut Vec<T>, range: Range<usize>, bytes: &[u8]) {
        (RkyvPatchesToVec { inner, seed, range }).apply(bytes)
    }

    #[inline(always)]
    pub fn apply(self, bytes: &[u8]) {
        let mut cur = self.seed;
        let patches = unsafe { rkyv::util::archived_root::<Vec<ArchivablePatch>>(&bytes) };

        let mut i = 0;

        for patch in patches.as_slice() {
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

    pub fn apply(val: &'a mut T, limit: usize, bytes: &[u8]) {
        ApplyRkyvPatches(val, limit).run(bytes)
    }

    pub fn run(self, bytes: &[u8]) {
        let mut i = 0;
        let patches = unsafe { rkyv::util::archived_root::<Vec<ArchivablePatch>>(&bytes) };

        for patch in patches.as_slice() {
            unsafe { apply_rkyv_patch(self.0, patch) };
            if i >= self.1 {
                break;
            }

            i += 1;
        }
    }
}
