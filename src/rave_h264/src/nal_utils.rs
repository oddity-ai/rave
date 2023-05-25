use bytes::Bytes;

use crate::error::Error;

type Result<T> = std::result::Result<T, Error>;

/// Split raw data into NALs.
///
/// # Return value
//
/// [`Vec`] of bytes for each NAL, or an error if the passed data does not start with a valid NAL
/// unit.
pub fn split_nals_annex_b(mut data: Bytes) -> Result<Vec<Bytes>> {
    const NAL_HEADER_1: [u8; 3] = [0x00, 0x00, 0x01];
    const NAL_HEADER_2: [u8; 4] = [0x00, 0x00, 0x00, 0x01];

    let first_offset = {
        if data.len() >= 3 && data[0..3] == NAL_HEADER_1 {
            3
        } else if data.len() >= 4 && data[0..4] == NAL_HEADER_2 {
            4
        } else {
            return Err(Error::H264AnnexBStartCodeMissing);
        }
    };

    data = data.slice(first_offset..);

    #[inline(always)]
    fn next(data: &mut Bytes) -> Bytes {
        for i in 0..data.len() {
            if (data.len() - i) >= 3 && data[i..i + 3] == NAL_HEADER_1 {
                let nal = data.split_to(i + 3);
                return nal.slice(0..nal.len() - 3);
            } else if (data.len() - 1) >= 4 && data[i..i + 4] == NAL_HEADER_2 {
                let nal = data.split_to(i + 4);
                return nal.slice(0..nal.len() - 4);
            }
        }

        data.clone()
    }

    let mut nals = Vec::new();
    while !data.is_empty() {
        nals.push(next(&mut data));
    }

    Ok(nals)
}
