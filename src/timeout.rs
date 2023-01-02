#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeoutTag(u32); // TODO: use `u16` or `i32` as they are safer than `u32` when using with WebAssembly

impl TimeoutTag {
    pub const fn new(tag: u32) -> Self {
        Self(tag)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeoutId(u64);

impl TimeoutId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub fn increment(&mut self) -> Self {
        let id = *self;
        self.0 += 1;
        id
    }
}
