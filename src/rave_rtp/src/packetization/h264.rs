// TODO: mixed use of NAL and NALU
use crate::error::Error;
use crate::packet::Packet;
use crate::packetization::common::{PacketizationParameters, Packetizer};

use bytes::{Buf, BufMut, Bytes, BytesMut};

type Result<T> = std::result::Result<T, Error>;

/// RTP H264 packetizer.
pub struct H264Packetizer {
    inner: Box<dyn H264Packetize>,
}

impl H264Packetizer {
    /// Create a new packetizer to create RTP packets from H264 encoded packets.
    ///
    /// # Packetization mode support
    ///
    /// The packetization modes currently supported are "Single NAL Unit mode" and "Non-Interleaved
    /// Mode".
    ///
    /// # Arguments
    ///
    /// * `mode` - Packetizer mode to use.
    /// * `params` - RTP Packetization parameters to use for constructing packets.
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

    /// Packetize one or more H264 encoded packets.
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
    /// If the packetizer is in single NAL unit mode, any data that exceed the MTU will produce an
    /// error. If the packetizer is in non-interleaved Mode, any data that exceed the MTU will be
    /// fragmented over multiple packets.
    ///
    /// # Arguments
    ///
    /// * `data` - One or more H264 packets.
    /// * `timestamp` - Presentation timestamp of NAL units.
    ///
    /// # Return value
    ///
    /// Zero or more RTP packets.
    #[inline]
    pub fn packetize(&mut self, data: Vec<Bytes>, timestamp: u32) -> Result<Vec<Packet>> {
        self.inner.packetize(data, timestamp)
    }
}

pub trait H264Packetize {
    /// Packetize one or more H264 encoded packets.
    ///
    /// # Access unit
    ///
    /// The caller must call this function exactly once per "access unit" (once per encoded
    /// picture).
    ///
    /// # Arguments
    ///
    /// * `data` - One or more H264 packets.
    /// * `timestamp` - Presentation timestamp of NAL units.
    ///
    /// # Return value
    ///
    /// Zero or more RTP packets.
    ///
    /// No packets may be returned even if valid data was passed. More than one packet may be
    /// produced if the data is fragmented over multiple packets to fit within the configured MTU.
    fn packetize(&mut self, data: Vec<Bytes>, timestamp: u32) -> Result<Vec<Packet>>;
}

/// Single NAL unit mode H264 packetizer.
#[derive(Debug)]
pub struct H264PacketizerMode0 {
    inner: Packetizer,
}

impl H264PacketizerMode0 {
    /// Create new H264 packetizer that packetizes in single NAL unit mode.
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
    /// Packetize one or more H264 encoded packets in single NAL unit mode.
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
    /// Since single NAL unit mode does not support fragmented MTUs. Any packets that exceed the MTU
    /// (if specified) will produce an error.
    ///
    /// # Arguments
    ///
    /// * `data` - One or more H264 packets.
    /// * `timestamp` - Presentation timestamp of NAL units.
    ///
    /// # Return value
    ///
    /// Zero or more RTP packets.
    fn packetize(&mut self, data: Vec<Bytes>, timestamp: u32) -> Result<Vec<Packet>> {
        let marker = false; // TODO: ?
        data.into_iter()
            .map(|nal| self.inner.packetize(nal, timestamp, marker))
            .collect()
    }
}

/// Non-Interleaved Mode H264 packetizer.
#[derive(Debug)]
pub struct H264PacketizerMode1 {
    inner: Packetizer,
    mtu: Option<usize>,
}

impl H264PacketizerMode1 {
    /// Create new H264 packetizer that packetizes in non-interleaved mode.
    ///
    /// # Arguments
    ///
    /// * `params` - Common RTP packetization parameters to use.
    pub fn new(params: PacketizationParameters) -> Self {
        let mtu = params.mtu;
        Self {
            inner: Packetizer::from_packetization_parameters(params),
            mtu,
        }
    }

    /// Groups a set of NALs such that packets that as much packets as possible are fit into a
    /// single STAP-A without exceeding the MTU.
    ///
    /// # Arguments
    ///
    /// * `data` - One ore more H264 packets to group.
    /// * `mtu` - Maximum transmission unit size.
    ///
    /// # Return value
    ///
    /// Groups of NALs.
    fn group_data_for_stap_a(&self, data: Vec<Bytes>, mtu: usize) -> Vec<Vec<Bytes>> {
        let mut grouped: Vec<Vec<Bytes>> = Vec::new();
        for nal in data {
            if let Some(current_group) = grouped.last_mut() {
                let combined_size = self.inner.header_serialized_len()
                    + current_group.iter().map(|nal| 2 + nal.len()).sum::<usize>();
                if combined_size <= mtu {
                    current_group.push(nal);
                } else {
                    grouped.push(vec![nal]);
                }
            } else {
                grouped.push(vec![nal]);
            }
        }

        grouped
    }
}

impl H264Packetize for H264PacketizerMode1 {
    /// Packetize one or more H264 encoded packets in non-interleaved mode.
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
    /// * `data` - One or more H264 packets.
    /// * `timestamp` - Presentation timestamp of NAL units.
    ///
    /// # Return value
    ///
    /// Zero or more packets.
    fn packetize(&mut self, data: Vec<Bytes>, timestamp: u32) -> Result<Vec<Packet>> {
        if let Some(mtu) = self.mtu {
            let mut packets: Vec<Packet> = Vec::new();
            for group in self.group_data_for_stap_a(data, mtu) {
                if group.len() == 1 {
                    let single_nal = group.into_iter().next().unwrap();
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
                        stap_a_payload(group)?,
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
                    .packetize(stap_a_payload(data)?, timestamp, false /* TODO? */)?;
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

// TODO: resequencing here (or maybe somewhere else?)

#[derive(Debug)]
pub struct H264Depacketizer {
    fragmented_unit_buffer: Option<BytesMut>,
}

impl H264Depacketizer {
    /// TODO
    pub fn new() -> Self {
        Self {
            fragmented_unit_buffer: None,
        }
    }

    /// Depacketize RTP packets and convert back to raw H264 NAL units that can be passed to a
    /// decoder.
    ///
    /// This function will reconstruct fragmented NALUs, as well as split aggregation packets back
    /// into separate H264 NAL units.
    ///
    /// # Packetization mode support
    ///
    /// The packetization modes currently supported are "Single NAL Unit mode" and "Non-Interleaved
    /// Mode".
    ///
    /// # Arguments
    ///
    /// * `packet` - RTP packet to depacketize.
    ///
    /// # Return value
    ///
    /// Zero or more depacketized NALs ready for decoding.
    ///
    /// No NAL units may be produced if the packet contains part of a fragmented unit. More packets
    /// may be produced if the RTP packet payload is an aggregation packet (STAP or MTAP).
    pub fn depacketize(&mut self, packet: &Packet) -> Result<Vec<Bytes>> {
        if packet.payload.len() <= 1 {
            return Err(Error::H264NalLengthTooSmall {
                len: packet.payload.len(),
            });
        }

        let nalu_type = packet.payload[0] & 0x1f;
        match nalu_type {
            // NAL
            1..=23 => {
                // This is just a normal NAL and can be passed on to the decoder as is.
                Ok(vec![packet.payload.clone()])
            }
            // STAP-A
            24 => {
                let mut payload = packet.payload.clone();
                payload.advance(1); // Skip NAL type (already peeked in nalu_type).

                std::iter::from_fn(|| {
                    if !payload.is_empty() {
                        if payload.remaining() < 2 {
                            return Some(Err(Error::H264AggregationUnitHeaderInvalid {
                                len: payload.remaining(),
                            }));
                        }
                        let nal_length = payload.get_u16() as usize;
                        if payload.remaining() < nal_length {
                            return Some(Err(Error::H264AggregationUnitDataTooSmall {
                                have: payload.remaining(),
                                need: nal_length,
                            }));
                        }
                        Some(Ok(payload.copy_to_bytes(nal_length)))
                    } else {
                        None
                    }
                })
                .collect()
            }
            // STAP-B
            25 => {
                // STAP-B only supported in packetization mode 2 (not supported here).
                Err(Error::H264DepacketizationNalTypeUnsupported {
                    nalu_type_name: "STAP-B".to_string(),
                })
            }
            // MTAP
            26..=27 => {
                // MTAP only supported in packetization mode 2 (not supported here).
                Err(Error::H264DepacketizationNalTypeUnsupported {
                    nalu_type_name: "MTAP".to_string(),
                })
            }
            // FU-A
            28 => {
                let mut payload = packet.payload.clone();
                payload.advance(1); // Skip NAL type (already peeked in nalu_type).

                if payload.remaining() < 1 {
                    return Err(Error::H264FragmentationUnitHeaderInvalid { len: payload.len() });
                }

                let fragmentation_unit_header = payload.get_u8();
                let start = (fragmentation_unit_header & 0x80) > 0;
                let end = (fragmentation_unit_header & 0x40) > 0;

                let recovered_nalu_payload = {
                    if start && !end {
                        if self.fragmented_unit_buffer.is_some() {
                            return Err(Error::H264FragmentedStateAlreadyStarted);
                        }
                        let mut fragmented_unit_buffer = BytesMut::new();
                        fragmented_unit_buffer.put(payload);
                        self.fragmented_unit_buffer = Some(fragmented_unit_buffer);
                        None
                    } else if !start && !end {
                        if let Some(fragmented_unit_buffer) = self.fragmented_unit_buffer.as_mut() {
                            fragmented_unit_buffer.put(payload);
                        } else {
                            return Err(Error::H264FragmentedStateNeverStarted);
                        }
                        None
                    } else if !start && end {
                        if let Some(mut fragmented_unit_buffer) = self.fragmented_unit_buffer.take()
                        {
                            fragmented_unit_buffer.put(payload);
                            Some(fragmented_unit_buffer.freeze())
                        } else {
                            return Err(Error::H264FragmentedStateNeverStarted);
                        }
                    } else {
                        // FU-A with start AND end bit set is just one unit (maybe it is illegal).
                        Some(payload)
                    }
                };

                if let Some(recovered_nalu_payload) = recovered_nalu_payload {
                    let nal_ref_idc = nalu_type & 0x60; // Copy original ref idc.
                    let nalu_type = fragmentation_unit_header & 0x1f;
                    let nalu_type = nalu_type | nal_ref_idc; // Recover original NALU type.
                    let mut nalu = BytesMut::new();
                    nalu.put_u8(nalu_type);
                    nalu.put(recovered_nalu_payload);
                    Ok(vec![nalu.freeze()])
                } else {
                    Ok(Vec::new())
                }
            }
            // FU-B
            29 => {
                // FU-B only supported in packetization mode 2 (not supported here).
                Err(Error::H264DepacketizationNalTypeUnsupported {
                    nalu_type_name: "FU-B".to_string(),
                })
            }
            // reserved
            30..=31 => {
                // RFC dictates that these must be ignored.
                Ok(Vec::new())
            }
            _ => Err(Error::H264DepacketizationNalTypeUnknown { nalu_type }),
        }
    }
}

/// H264 packetization mode.
///
/// The following table (from RFC 6184) specifies which payload types are supported per
/// packetization mode:
///
/// ```text
/// Payload Packet    Single NAL    Non-Interleaved    Interleaved
/// Type    Type      Unit Mode           Mode             Mode
/// -------------------------------------------------------------
/// 0      reserved      ig               ig               ig
/// 1-23   NAL unit     yes              yes               no
/// 24     STAP-A        no              yes               no
/// 25     STAP-B        no               no              yes
/// 26     MTAP16        no               no              yes
/// 27     MTAP24        no               no              yes
/// 28     FU-A          no              yes              yes
/// 29     FU-B          no               no              yes
/// 30-31  reserved      ig               ig               ig
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum H264PacketizationMode {
    /// Single NAL unit mode.
    ///
    /// Targeted for conversational systems.
    SingleNalUnit,
    /// Non-interleaved mode.
    ///
    /// NAL units are transmitted in NAL unit decoding order. Targeted for systems that do not
    /// require very low end-to-end latency.
    NonInterleavedMode,
    /// Interleaved mode.
    ///
    /// Allows transmission of NAL units out of NAL unit decoding order.
    InterleavedMode,
}

impl TryFrom<usize> for H264PacketizationMode {
    type Error = Error;

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
