use crate::ProgramId;
use sha2::{Digest, Sha256};
use std::io::{self, Cursor};

// lz4 brings the empty save sizes from >1MiB each down to 4-5KiB so its worth it

pub const V4_LZ4: &[u8] = include_bytes!("./v4.lz4");
pub const V5_LZ4: &[u8] = include_bytes!("./v5.lz4");

pub fn get_decompressed(version: u8) -> Vec<u8> {
    use lz4_flex::frame::FrameDecoder;

    let sav = match version {
        0..=4 => V4_LZ4,
        5 => V5_LZ4,
        _ => unimplemented!(),
    };

    let mut buf = Vec::new();
    let mut dec = FrameDecoder::new(sav);
    io::copy(&mut dec, &mut Cursor::new(&mut buf)).unwrap();

    buf
}

pub struct Savefile(pub Vec<u8>);

impl Savefile {
    pub fn new(version: u8, saveid: ProgramId) -> Self {
        let mut this = Self(get_decompressed(version));
        this.patch_saveid(saveid);
        this
    }

    pub fn patch_saveid(&mut self, saveid: ProgramId) -> &mut Self {
        // patch saveids in Extra Data A and B
        for offset in [0x6D8 + 0x18, 0x8D8 + 0x18] {
            let span = &mut self.0[offset..(offset + 0x8)];
            span.copy_from_slice(&saveid.to_le_bytes());
        }

        // fix master hash in the DISF header
        let hash = Sha256::digest(&self.0[0x300..0x4000]);
        let hash_location = &mut self.0[0x100 + 0x8..0x100 + 0x8 + 0x20];
        hash_location.copy_from_slice(hash.as_ref());

        // DISF hash is now broken but hactoolnet fixes that for us

        self
    }

    pub fn to_reader(&mut self) -> Cursor<&mut Vec<u8>> {
        Cursor::new(&mut self.0)
    }
}

impl AsRef<[u8]> for Savefile {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}
