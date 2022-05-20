use serde::de::{DeserializeSeed, Deserializer, SeqAccess, Visitor};
use serde::{
    ser::{SerializeSeq, Serializer},
    Serialize,
};
use std::ops::Deref;

use std::fmt;

pub use vhs_diff_macros::{Diff, Patch};

pub mod patch_seq;
pub mod rkyv_seq;

pub trait Patch {
    fn do_patch_command<'de, D>(
        &mut self,
        field_index: u8,
        deserializer: D,
    ) -> Result<(), D::Error>
    where
        D: Deserializer<'de>;

    // applies a patch from a serde SeqAccess
    fn do_patch_from_seq<'de, A>(&mut self, field_index: u8, seq: &mut A) -> Result<(), A::Error>
    where
        A: SeqAccess<'de>;

    unsafe fn do_patch_from_byteslice<D>(
        &mut self,
        field_index: u8,
        deser: &mut D,
        position: usize,
        bytes: &[u8],
    ) -> Result<(), D::Error>
    where
        D: rkyv::Fallible + ?Sized;
}

pub struct PatchDeserializer<'a, T: 'a>(&'a mut T);

impl<'a, T: Patch> PatchDeserializer<'a, T> {
    pub fn new(val: &'a mut T) -> Self {
        PatchDeserializer(val)
    }

    pub fn apply<'de, D>(val: &'a mut T, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        PatchDeserializer(val).deserialize(deserializer)
    }
}

pub trait Diff {
    fn diff(&self, rhs: Self) -> OwnedPatch;

    fn diff_rkyv(&self, rhs: Self) -> ArchivablePatch;
}

impl<'de, 'a, T> DeserializeSeed<'de> for PatchDeserializer<'a, T>
where
    T: Patch,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PatchVisitor<'a, T: 'a>(&'a mut T);

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
                while let Some(field_index) = seq.next_element::<u8>()? {
                    self.0.do_patch_from_seq(field_index, &mut seq)?;
                }

                Ok(())
            }
        }

        deserializer.deserialize_seq(PatchVisitor(self.0))
    }
}

pub struct OwnedDiffCommand {
    pub index: u8,
    pub value: Box<dyn erased_serde::Serialize>,
}

pub struct OwnedPatch(pub Vec<OwnedDiffCommand>);

impl Deref for OwnedPatch {
    type Target = Vec<OwnedDiffCommand>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for OwnedPatch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len() * 2))?;
        for patch in &self.0 {
            seq.serialize_element(&patch.index)?;
            seq.serialize_element(&patch.value)?;
        }

        seq.end()
    }
}

#[allow(unused_must_use)]
#[inline(always)]
pub unsafe fn apply_rkyv_patch<T: Patch>(start: &mut T, patch: &ArchivedArchivablePatch) {
    let mut deser = rkyv::Infallible;
    let bytes = patch.patch_bytes.as_slice();
    for index in patch.field_positions.as_slice().iter() {
        start.do_patch_from_byteslice(
            index.index,
            &mut deser,
            index.position.value() as usize,
            bytes,
        );
    }
}

#[derive(rkyv::Serialize, rkyv::Archive, rkyv::Deserialize)]
pub struct ArchivablePatch {
    pub field_positions: Vec<FieldAndPosition>,
    pub patch_bytes: Vec<u8>,
}

#[derive(Clone, Copy, rkyv::Serialize, rkyv::Archive, rkyv::Deserialize)]
#[archive_attr(repr(C, align(8)))]
pub struct FieldAndPosition {
    pub index: u8,
    pub position: u32,
}

#[derive(rkyv::Serialize, rkyv::Archive, rkyv::Deserialize)]
pub struct ArchivablePatchSeq {
    pub base: Vec<u8>,
    pub patches: Vec<ArchivablePatch>,
}

impl ArchivablePatchSeq {
    pub fn from_base_and_patches<T>(base: &T, patches: Vec<ArchivablePatch>) -> ArchivablePatchSeq
    where
        T: rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<1024>>,
    {
        ArchivablePatchSeq {
            base: rkyv::to_bytes::<_, 1024>(base).unwrap().to_vec(),
            patches,
        }
    }
}
