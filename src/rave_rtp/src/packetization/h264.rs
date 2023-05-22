use crate::error::Error;
use crate::packet::Packet;
use crate::packetization::common::{PacketizationParameters, Packetizer};

use bytes::{BufMut, Bytes, BytesMut};

type Result<T> = std::result::Result<T, Error>;

/// RTP H264 packetizer.
pub struct H264Packetizer {
    inner: Box<dyn H264Packetize>,
}

impl H264Packetizer {
    /// Create a new packetizer to create RTP packets with H264 encoded pictures.
    ///
    /// # Arguments
    ///
    /// * `mode` - Packetizer mode to use.
    /// * `params` - RTP Packetization parameters to use for constructing packets.
    ///
    /// # Packetization mode support
    ///
    /// The packetization modes currently supported are "Single NAL Unit mode" and "Non-Interleaved
    /// Mode".
    pub fn from_packetization_mode(
        mode: H264PacketizationMode,
        params: PacketizationParameters,
    ) -> Result<Self> {
        Ok(Self {
            inner: match mode {
                H264PacketizationMode::SingleNalUnit => Box::new(H264PacketizerMode0::new(params)),
                H264PacketizationMode::NonInterleavedMode => {
                    Box::new(H264PacketizerMode1::new(params))
                }
                H264PacketizationMode::InterleavedMode => {
                    return Err(Error::H264PacketizationModeUnsupported { mode })
                }
            },
        })
    }

    /// Packetize one or more H264 NAL units.
    ///
    /// Refer to [`H264Packetize::packetize()`].
    ///
    /// # Access unit
    ///
    /// The caller must call this function exactly once per "access unit" (once per encoded
    /// picture).
    ///
    /// # Fragmentation
    ///
    /// If the packetizer is in Single NAL Unit mode, any data that exceed the MTU will produce an
    /// error. If the packetizer is in Non-Interleaved Mode, any data that exceed the MTU will be
    /// fragmented over multiple packets.
    ///
    /// # Arguments
    ///
    /// * `data` - Data containing one or more H264 packets (serialzied NAL units).
    /// * `timestamp` - Presentation timestamp of NAL units.
    ///
    /// # Return value
    ///
    /// Zero or more packets.
    #[inline]
    pub fn packetize(&mut self, data: Bytes, timestamp: u32) -> Result<Vec<Packet>> {
        self.inner.packetize(data, timestamp)
    }
}

pub trait H264Packetize {
    /// Packetize one or more H264 NAL units.
    ///
    /// # Access unit
    ///
    /// The caller must call this function exactly once per "access unit" (once per encoded
    /// picture).
    ///
    /// # Arguments
    ///
    /// * `data` - Data containing one or more H264 packets (serialzied NAL units).
    /// * `timestamp` - Presentation timestamp of NAL units.
    ///
    /// # Return value
    ///
    /// Zero or more packets.
    ///
    /// No packets may be returned even if valid data was passed. More than one packet may be
    /// produced if the data is fragmented over multiple packets to fit within the configured MTU.
    fn packetize(&mut self, data: Bytes, timestamp: u32) -> Result<Vec<Packet>>;
}

/// Single NAL Unit Mode H264 packetizer.
#[derive(Debug)]
pub struct H264PacketizerMode0 {
    inner: Packetizer,
}

impl H264PacketizerMode0 {
    /// Create new H264 packetizer that packetizes in Single NAL Unit mode.
    ///
    /// # Arguments
    ///
    /// * `params` - Common RTP packetization parameters to use.
    pub fn new(params: PacketizationParameters) -> Self {
        Self {
            inner: Packetizer::from_packetization_parameters(params),
        }
    }
}

impl H264Packetize for H264PacketizerMode0 {
    /// Packetize one or more H264 NAL units in Single NAL Unit mode.
    ///
    /// Refer to [`H264Packetize::packetize()`].
    ///
    /// # Access unit
    ///
    /// The caller must call this function exactly once per "access unit" (once per encoded
    /// picture).
    ///
    /// # MTU
    ///
    /// Since Single NAL Unit mode does not support fragmented MTUs, any packets that exceed the MTU
    /// (if specified) will produce an error.
    ///
    /// # Arguments
    ///
    /// * `data` - Data containing one or more H264 packets (serialzied NAL units).
    /// * `timestamp` - Presentation timestamp of NAL units.
    ///
    /// # Return value
    ///
    /// Zero or more packets.
    fn packetize(&mut self, data: Bytes, timestamp: u32) -> Result<Vec<Packet>> {
        let marker = false; // TODO: ?
        split_nals(data)?
            .into_iter()
            .map(|nal| self.inner.packetize(nal, timestamp, marker))
            .collect()
    }
}

/// Non-Interleaved Mode H264 packetizer.
#[derive(Debug)]
pub struct H264PacketizerMode1 {
    inner: Packetizer,
    mtu: Option<usize>, // TODO: fragment if payload higher than MTU
}

impl H264PacketizerMode1 {
    /// Create new H264 packetizer that packetizes in Non-Interleaved mode.
    ///
    /// # Arguments
    ///
    /// * `params` - Common RTP packetization parameters to use.
    pub fn new(params: PacketizationParameters) -> Self {
        let mtu = params.mtu.clone();
        Self {
            inner: Packetizer::from_packetization_parameters(params),
            mtu,
        }
    }
}

impl H264Packetize for H264PacketizerMode1 {
    /// Packetize one or more H264 NAL units in Non-Interleaved Mode.
    ///
    /// Refer to [`H264Packetize::packetize()`].
    ///
    /// # Access unit
    ///
    /// The caller must call this function exactly once per "access unit" (once per encoded
    /// picture).
    ///
    /// # Fragmentation
    ///
    /// Data may be fragmented over multiple packets to satisfy MTU.
    ///
    /// # Arguments
    ///
    /// * `data` - Data containing one or more H264 packets (serialzied NAL units).
    /// * `timestamp` - Presentation timestamp of NAL units.
    ///
    /// # Return value
    ///
    /// Zero or more packets.
    fn packetize(&mut self, data: Bytes, timestamp: u32) -> Result<Vec<Packet>> {
        let nals = split_nals(data)?;

        if let Some(mtu) = self.mtu {
            let mut buckets: Vec<Vec<Bytes>> = Vec::new();
            for nal in nals {
                if let Some(current_bucket) = buckets.last_mut() {
                    let combined_size = self.inner.header_serialized_len()
                        + current_bucket
                            .iter()
                            .map(|nal| 2 + nal.len())
                            .sum::<usize>();
                    if combined_size <= mtu {
                        current_bucket.push(nal);
                    } else {
                        buckets.push(vec![nal]);
                    }
                } else {
                    buckets.push(vec![nal]);
                }
            }

            let mut packets: Vec<Packet> = Vec::new();
            for bucket in buckets {
                if bucket.len() == 1 {
                    let single_nal = bucket.into_iter().next().unwrap();
                    if (self.inner.header_serialized_len() + single_nal.len()) <= mtu {
                        let single_nal_packet = self
                            .inner
                            .packetize(single_nal, timestamp, false /* TODO */)?;
                        packets.push(single_nal_packet);
                    } else {
                        // TODO: fragment!!!!
                    }
                } else {
                    let stap_a_packet = self.inner.packetize(
                        stap_a_payload(bucket)?,
                        timestamp,
                        false, /* TODO */
                    )?;
                    packets.push(stap_a_packet);
                }
            }

            Ok(packets)
        } else {
            let stap_a_packet =
                self.inner
                    .packetize(stap_a_payload(nals)?, timestamp, false /* TODO? */)?;
            Ok(vec![stap_a_packet])
        }
    }
}

// TODO
fn stap_a_payload(nals: Vec<Bytes>) -> Result<Bytes> {
    let mut payload = BytesMut::new();
    for nal in nals {
        payload.put_u16(
            nal.len()
                .try_into()
                .map_err(|_| Error::H264NalDataLengthInvalid { len: nal.len() })?,
        );
        payload.put(nal);
    }

    Ok(payload.into())
}

/// Split raw data into NALs.
///
/// # Return value
///
/// [`Vec`] of bytes for each NAL, or an error if the passed data does not start with a valid NAL
/// unit.
fn split_nals(mut data: Bytes) -> Result<Vec<Bytes>> {
    const NAL_HEADER_1: [u8; 3] = [0x00, 0x00, 0x01];
    const NAL_HEADER_2: [u8; 4] = [0x00, 0x00, 0x00, 0x01];

    let offset = if data.len() >= 3 && data[0..3] == NAL_HEADER_1 {
        3
    } else if data.len() >= 4 && data[0..4] == NAL_HEADER_2 {
        4
    } else {
        return Err(Error::H264InvalidNalHeader);
    };

    let mut nals = Vec::new();
    for i in offset..data.len() {
        if ((data.len() - i) >= 3 && data[i..i + 3] == NAL_HEADER_1)
            || ((data.len() - 1) >= 4 && data[i..i + 4] == NAL_HEADER_2)
        {
            nals.push(data.split_to(i));
        }
    }

    nals.push(data);

    Ok(nals)
}

#[derive(Debug)]
pub struct H264Depacketizer {
    // TODO
}

impl H264Depacketizer {
    // TODO: this holds state like parameter sets as well

    /// Depacketize RTP packets and convert back to raw H264 NAL units that can be passed to a
    /// decoder.
    ///
    /// This function will reconstruct fragmented NALUs, as well as split aggregation packets back
    /// into separate H264 NAL units.
    ///
    /// # Arguments
    ///
    /// * `packet` - RTP Packet to depacketize.
    ///
    /// # Return value
    ///
    /// Zero or more H264 packets ready for decoding.
    ///
    /// No NAL units may be produced if the packet contains part of a fragmented unit. More packets
    /// may be produced if the RTP packet payload is an aggregation packet (STAP or MTAP).
    pub fn depacketize(&mut self, packet: &Packet) -> Vec<Bytes> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H264PacketizationMode {
    SingleNalUnit,
    NonInterleavedMode,
    InterleavedMode,
}

impl TryFrom<usize> for H264PacketizationMode {
    type Error = Error; // TODO: dummy

    fn try_from(mode: usize) -> Result<Self> {
        match mode {
            0 => Ok(H264PacketizationMode::SingleNalUnit),
            1 => Ok(H264PacketizationMode::NonInterleavedMode),
            2 => Ok(H264PacketizationMode::InterleavedMode),
            _ => Err(Error::H264PacketizationModeUnknown { mode }),
        }
    }
}

impl std::fmt::Display for H264PacketizationMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            H264PacketizationMode::SingleNalUnit => write!(f, "single nal unit"),
            H264PacketizationMode::NonInterleavedMode => write!(f, "non-interleaved mode"),
            H264PacketizationMode::InterleavedMode => write!(f, "interleaved mode"),
        }
    }
}
