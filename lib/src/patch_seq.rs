pub use super::*;
use serde::de::{SeqAccess, Visitor};
use std::ops::Range;

pub struct PatchesToVec<'a, T: 'a> {
    inner: &'a mut Vec<T>,
    seed: T,
    range: Range<usize>,
}

impl<'a, T: Patch + Clone> PatchesToVec<'a, T> {
    pub fn new(seed: T, inner: &'a mut Vec<T>) -> PatchesToVec<T> {
        PatchesToVec {
            inner,
            seed,
            range: usize::MIN..usize::MAX,
        }
    }

    pub fn new_range(seed: T, inner: &'a mut Vec<T>, range: Range<usize>) -> PatchesToVec<T> {
        PatchesToVec { inner, seed, range }
    }

    pub fn apply_range<'de, D>(
        seed: T,
        inner: &'a mut Vec<T>,
        range: Range<usize>,
        deserializer: D,
    ) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        (PatchesToVec { inner, seed, range }).deserialize(deserializer)
    }
}

impl<'de, 'a, T> DeserializeSeed<'de> for PatchesToVec<'a, T>
where
    T: Patch + Clone,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PatchesToVecVisitor<'a, T: 'a> {
            inner: &'a mut Vec<T>,
            seed: T,
            range: Range<usize>,
        }

        impl<'de, 'a, T> Visitor<'de> for PatchesToVecVisitor<'a, T>
        where
            T: Patch + Clone,
        {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an array of patch arrays")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut cur = self.seed;

                let mut i = 0;

                while let Some(()) = seq.next_element_seed(PatchDeserializer::new(&mut cur))? {
                    if i >= self.range.end {
                        break;
                    }

                    if self.range.contains(&i) {
                        self.inner.push(cur.clone());
                    }

                    i += 1;
                }

                Ok(())
            }
        }

        deserializer.deserialize_seq(PatchesToVecVisitor {
            seed: self.seed,
            range: self.range,
            inner: self.inner,
        })
    }
}

// applies a sequence of patches to an object, up to an index
pub struct ApplyPatches<'a, T: 'a>(&'a mut T, usize);

impl<'a, T: Patch> ApplyPatches<'a, T> {
    pub fn new(val: &'a mut T, limit: usize) -> Self {
        ApplyPatches(val, limit)
    }

    pub fn apply<'de, D>(val: &'a mut T, limit: usize, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        ApplyPatches(val, limit).deserialize(deserializer)
    }
}

impl<'de, 'a, T> DeserializeSeed<'de> for ApplyPatches<'a, T>
where
    T: Patch,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PatchVisitor<'a, T: 'a>(&'a mut T, usize);

        impl<'de, 'a, T> Visitor<'de> for PatchVisitor<'a, T>
        where
            T: Patch,
        {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an array of u8s followed by a dynamic value")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut i = 0;

                while let Some(()) = seq.next_element_seed(PatchDeserializer::new(self.0))? {
                    if i >= self.1 {
                        break;
                    }

                    i += 1;
                }

                Ok(())
            }
        }

        deserializer.deserialize_seq(PatchVisitor(self.0, self.1))
    }
}
