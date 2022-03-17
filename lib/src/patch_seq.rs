pub use super::*;

use serde::de::{SeqAccess, Visitor};

pub struct PatchesToVec<T>(T, usize);

impl<T: Patch + Clone> PatchesToVec<T> {
    pub fn new(val: T) -> PatchesToVec<T> {
        PatchesToVec(val, usize::MAX)
    }

    pub fn apply_n(val: T, limit: usize) -> PatchesToVec<T> {
        PatchesToVec(val, limit)
    }
}

impl<'de, T> Visitor<'de> for PatchesToVec<T>
where
    T: Patch + Clone,
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "an array of patch arrays")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Vec<T>, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        let mut cur = self.0;

        while let Some(()) = seq.next_element_seed(PatchDeserializer::new(&mut cur))? {
            if vec.len() >= self.1 {
                break;
            }

            vec.push(cur.clone());
        }

        Ok(vec)
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
