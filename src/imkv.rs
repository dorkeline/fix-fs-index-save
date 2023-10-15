//! Types and methods for the IMKV ("imkvdb.arc") key-value stores found in FS and NCM saves.

use binrw::binrw;
use core::fmt;

#[binrw]
#[brw(little, magic = b"IMEN")]
#[derive(Default, Debug, Clone)]
pub struct Imen {
    key_len: u32,
    value_len: u32,

    #[br(count = key_len)]
    key: Vec<u8>,

    #[br(count = value_len)]
    value: Vec<u8>,
}

impl Imen {
    pub fn from_kv(key: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>) -> Self {
        let (key, value) = (key.into(), value.into());

        assert!(key.len() <= u32::MAX as usize);
        assert!(value.len() <= u32::MAX as usize);

        Self {
            key_len: key.len() as u32,
            value_len: value.len() as u32,
            key,
            value,
        }
    }

    pub fn key(&self) -> &Vec<u8> {
        &self.key
    }

    pub fn value(&self) -> &Vec<u8> {
        &self.value
    }

    fn map_attr(attr: &mut Vec<u8>, size: &mut u32, f: impl FnOnce(&mut Vec<u8>)) -> u32 {
        f(attr);
        assert!(attr.len() <= u32::MAX as usize);
        *size = attr.len() as u32;
        *size
    }

    pub fn map_key(&mut self, f: impl FnOnce(&mut Vec<u8>)) -> u32 {
        Self::map_attr(&mut self.key, &mut self.key_len, f)
    }

    pub fn map_value(&mut self, f: impl FnOnce(&mut Vec<u8>)) -> u32 {
        Self::map_attr(&mut self.value, &mut self.value_len, f)
    }
}

fn to_hex(v: &[u8]) -> String {
    use fmt::Write;

    let mut s = String::with_capacity(v.len() * 2);

    for b in v {
        write!(&mut s, "{:02x}", b).ok();
    }

    s
}

impl fmt::Display for Imen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "IMEN\n  Key (0x{:x}):\t{}\n  Value (0x{:x}):\t{}",
            self.key_len,
            to_hex(self.key.as_ref()),
            self.value_len,
            to_hex(self.value.as_ref())
        )
    }
}

#[binrw]
#[brw(little, magic = b"IMKV")]
#[derive(Default, Debug, Clone)]
pub struct Imkv {
    reserved: u32,
    entry_cnt: u32,

    #[br(count = entry_cnt)]
    entries: Vec<Imen>,
}

impl Imkv {
    pub fn insert(&mut self, key: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>) {
        self.entries.push(Imen::from_kv(key, value));
        self.entry_cnt += 1;
    }

    pub fn entries(&self) -> &Vec<Imen> {
        &self.entries
    }

    pub fn map_entries(&mut self, f: impl FnOnce(&mut Vec<Imen>)) -> u32 {
        f(&mut self.entries);
        assert!(self.entries.len() <= u32::MAX as usize);
        self.entry_cnt = self.entries.len() as u32;
        self.entry_cnt
    }
}

impl FromIterator<Imen> for Imkv {
    fn from_iter<T: IntoIterator<Item = Imen>>(imens: T) -> Self {
        let entries = Vec::from_iter(imens);

        assert!(entries.len() <= u32::MAX as usize);

        Self {
            reserved: 0,
            entry_cnt: entries.len() as u32,
            entries,
        }
    }
}
