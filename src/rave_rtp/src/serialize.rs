use bytes::{BufMut, BytesMut};

use crate::error::{Error, Result};
use crate::packet::Packet;

pub trait Serialize {
    fn serialize(&self, dst: &mut BytesMut) -> Result<()>;
}

impl Serialize for Packet {
    fn serialize(&self, dst: &mut BytesMut) -> Result<()> {
        dst.reserve(12 + (self.csrc.len() * 4)); // TODO: count extension
        let version = (self.version.as_number() as u8) << 6;
        let csrc_count: u8 = self
            .csrc
            .len()
            .try_into()
            .map_err(|_| Error::CsrcCountInvalid {
                count: self.csrc.len(),
            })?;
        let padding = if self.padding { 0x01_u8 } else { 0x00_u8 } << 5;
        let extension = if self.extension.is_some() {
            0x01_u8
        } else {
            0x00_u8
        } << 4;
        dst.put_u8(version | csrc_count | padding | extension);

        let marker = if self.marker { 0x01_u8 } else { 0x00_u8 } << 7;
        dst.put_u8(self.payload_type | marker);

        dst.put_u16(self.sequence_number);
        dst.put_u32(self.timestamp);
        dst.put_u32(self.ssrc);
        for csrc_item in self.csrc.iter() {
            dst.put_u32(*csrc_item);
        }

        if let Some(extension) = self.extension.as_ref() {
            dst.put_u16(extension.profile_identifier);
            dst.put_u16(extension.data.len().try_into().map_err(|_| {
                Error::ExtensionLengthInvalid {
                    length: extension.data.len(),
                }
            })?);
        }

        Ok(())
    }
}
