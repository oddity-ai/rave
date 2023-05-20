use bytes::{BufMut, BytesMut};

use crate::error::{Error, Result};
use crate::packet::{Extension, Header, Packet, PacketPadded};

pub trait Serialize {
    fn serialize(self, dst: &mut BytesMut) -> Result<()>;
    fn serialized_len(&self) -> usize;
}

impl Serialize for Packet {
    fn serialize(self, dst: &mut BytesMut) -> Result<()> {
        assert!(
            !self.header.padding,
            "header padding bit must be false when serializing packet without padding"
        );
        self.header.serialize(dst)?;
        dst.put(self.payload);
        Ok(())
    }

    fn serialized_len(&self) -> usize {
        self.header.serialized_len() + self.payload.len()
    }
}

impl Serialize for PacketPadded {
    fn serialize(self, dst: &mut BytesMut) -> Result<()> {
        assert!(
            self.packet.header.padding,
            "header padding bit must be true when serializing packet with padding",
        );
        self.packet.serialize(dst)?;
        let padding_len = calculate_padding(self.padding_divisor, dst.len())
            .try_into()
            .map_err(|_| Error::PaddingLengthInvalid {
                padding_divisor: self.padding_divisor,
                len: dst.len(),
            })?;
        dst.put_bytes(0x00_u8, (padding_len - 1) as usize);
        dst.put_u8(padding_len);
        Ok(())
    }

    fn serialized_len(&self) -> usize {
        let packet_len = self.packet.serialized_len();
        packet_len + calculate_padding(self.padding_divisor, packet_len)
    }
}

impl Serialize for Header {
    fn serialize(self, dst: &mut BytesMut) -> Result<()> {
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
        for csrc_item in self.csrc {
            dst.put_u32(csrc_item);
        }

        if let Some(extension) = self.extension {
            extension.serialize(dst)?;
        }

        Ok(())
    }

    fn serialized_len(&self) -> usize {
        12 + (self.csrc.len() * 4)
            + (self
                .extension
                .as_ref()
                .map(|extension| extension.serialized_len())
                .unwrap_or(0))
    }
}

impl Serialize for Extension {
    fn serialize(self, dst: &mut BytesMut) -> Result<()> {
        dst.put_u16(self.profile_identifier);
        dst.put_u16(
            self.data
                .len()
                .try_into()
                .map_err(|_| Error::ExtensionLengthInvalid {
                    len: self.data.len(),
                })?,
        );
        for data_item in self.data {
            dst.put_u32(data_item);
        }

        Ok(())
    }

    fn serialized_len(&self) -> usize {
        4 + (self.data.len() * 4)
    }
}

#[inline]
fn calculate_padding(padding_divisor: u8, len: usize) -> usize {
    (padding_divisor as usize) - (len % (padding_divisor as usize))
}
