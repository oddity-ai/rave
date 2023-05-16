pub use bytes::Buf;

const LN: u8 = b'\x0a';
const CR: u8 = b'\x0d';

pub trait ReadLine {
    fn read_line(&mut self) -> Option<Result<String, std::string::FromUtf8Error>>;
}

impl<T> ReadLine for T
where
    T: Buf,
{
    fn read_line(&mut self) -> Option<Result<String, std::string::FromUtf8Error>> {
        if self.remaining() == 0 {
            return None;
        }

        let mut found = false;
        let mut end = 0; // Index of LN, CR or CRLN
        let mut skip = 0; // Size of LN, CR or CRLN

        let chunk = self.chunk();

        for i in 0..chunk.len() - 1 {
            if chunk[i] == CR && chunk[i + 1] == LN {
                // Found CRLN at [i]
                (found, end, skip) = (true, i, 2);
                break;
            }

            if chunk[i] == CR || chunk[i] == LN {
                // Found CR or LN at [i]
                (found, end, skip) = (true, i, 1);
                break;
            }
        }

        if !found {
            let last = chunk.len() - 1;
            // Note that we explicitly do not check for CR here, since we can't know for sure if
            // there isn't another LN coming after because this is the last character in the buffer.
            if chunk[last] == LN {
                // Found CRat [i]
                (found, end, skip) = (true, last, 1);
            }
        }

        if found {
            let line = String::from_utf8(chunk[0..end].to_vec());
            self.advance(end + skip);
            Some(line)
        } else {
            None
        }
    }
}
