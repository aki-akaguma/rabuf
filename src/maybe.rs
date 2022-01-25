use std::ops::Deref;

/// this is buffer, but maybe slice.
#[derive(Debug, Clone)]
pub enum MaybeSlice<'a> {
    Slice(&'a [u8]),
    Buffer(Vec<u8>),
}

impl<'a> MaybeSlice<'a> {
    pub fn into_vec(self) -> Vec<u8> {
        match self {
            MaybeSlice::Slice(x) => x.to_vec(),
            MaybeSlice::Buffer(v) => v,
        }
    }
}

impl<'a> Deref for MaybeSlice<'a> {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &<Self as Deref>::Target {
        match self {
            MaybeSlice::Slice(x) => x,
            MaybeSlice::Buffer(v) => v,
        }
    }
}
