use core::fmt;

pub type ProgramId = HexU64;

#[binrw::binrw]
#[brw(little)]
#[derive(Clone)]
#[repr(transparent)]
pub struct HexU64(pub u64);

impl HexU64 {
    pub fn new(val: u64) -> Self {
        Self(val)
    }
}

impl fmt::Debug for HexU64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

impl fmt::Display for HexU64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }
}

impl core::ops::Deref for HexU64 {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u64> for HexU64 {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct SaveDataCreationInfo {
    size: HexU64,
    journal_size: HexU64,
    avail_size: HexU64,
    owner_id: ProgramId,
    flags: u32,
    space_id: u8,
    unk: u8,
    uninit: [u8; 0x1a],
}

#[binrw::binrw]
#[brw(little, repr = u8)]
#[derive(Debug, Clone)]
pub enum SaveDataType {
    System = 0,
    Account,
    Bcat,
    Device,
    Temporary,
    Cache,
    SystemBcat,
}

#[binrw::binrw]
#[brw(little, repr = u8)]
#[derive(Debug, Clone)]
pub enum SaveDataRank {
    Primary = 0,
    Secondary,
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct SaveDataAttribute {
    pub appid: ProgramId,
    pub userid: [u64; 2],
    pub system_save_data_id: ProgramId,
    pub save_type: SaveDataType,
    pub rank: SaveDataRank,
    pub index: u16,
    pub padding: u32,
    pub unk: [u64; 3],
}

impl SaveDataAttribute {
    pub fn new_system(system_save_data_id: impl Into<ProgramId>) -> Self {
        Self {
            appid: 0.into(),
            userid: [0, 0],
            system_save_data_id: system_save_data_id.into(),
            save_type: SaveDataType::System,
            rank: SaveDataRank::Primary,
            index: 0,
            padding: 0,
            unk: [0, 0, 0],
        }
    }
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct WhateverTheValueIs {
    pub saveid: ProgramId,
    pub save_size: u64,
    pub unknown: [u8; 0x30],
}
