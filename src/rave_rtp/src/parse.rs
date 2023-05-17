use bytes::{Buf, Bytes};

use crate::error::{Error, Result};
use crate::packet::{Extension, Header, Packet, Version};

pub trait Parse: Sized {
    fn parse(src: &mut Bytes) -> Result<Self>;
}

impl Parse for Packet {
    fn parse(src: &mut Bytes) -> Result<Self> {
        let header = Header::parse(src)?;
        let padding_len = if header.padding {
            *src.last()
                .ok_or(Error::NotEnoughData { have: 0, need: 1 })? as usize
        } else {
            0
        };

        if src.remaining() < padding_len {
            return Err(Error::NotEnoughData {
                have: src.remaining(),
                need: padding_len,
            });
        }

        let payload_len = src.remaining() - padding_len;
        let payload = src.copy_to_bytes(payload_len);

        Ok(Packet::new(header, payload))
    }
}

impl Parse for Header {
    fn parse(src: &mut Bytes) -> Result<Self> {
        let bytes_len = src.remaining();

        if bytes_len < 12 {
            return Err(Error::NotEnoughData {
                have: bytes_len,
                need: 12,
            });
        }

        let byte = src.get_u8();
        let version = Version::try_from((byte >> 6 & 0x03) as usize)?;
        let padding = (byte >> 5 & 0x01) > 0;
        let extension = (byte >> 4 & 0x01) > 0;
        let csrc_count = (byte & 0x0f) as usize;

        let need = 12 + (csrc_count * 4);
        if bytes_len < need {
            return Err(Error::NotEnoughData {
                have: bytes_len,
                need,
            });
        }

        let byte = src.get_u8();
        let marker = (byte >> 7 & 0x01) > 0;
        let payload_type = byte & 0x7f;
        let sequence_number = src.get_u16();
        let timestamp = src.get_u32();
        let ssrc = src.get_u32();

        let csrc = (0..csrc_count).map(|_| src.get_u32()).collect::<Vec<_>>();

        let extension = if extension {
            if src.remaining() < 4 {
                return Err(Error::NotEnoughData {
                    have: src.remaining(),
                    need: 4,
                });
            }

            let profile_identifier = src.get_u16();
            let len = src.get_u16();
            let need = len as usize * 4;
            if src.remaining() < need {
                return Err(Error::NotEnoughData {
                    have: src.remaining(),
                    need,
                });
            }

            let data = (0..len).map(|_| src.get_u32()).collect::<Vec<_>>();
            Some(Extension {
                profile_identifier,
                data,
            })
        } else {
            None
        };

        Ok(Header {
            version,
            padding,
            marker,
            payload_type,
            sequence_number,
            timestamp,
            ssrc,
            csrc,
            extension,
        })
    }
}
