use std::cmp::min;
use std::io;
use std::io::{Error, Read, Seek, SeekFrom, Write};

use num_format::{Locale, ToFormattedString};
use rand_core::RngCore;
use std::sync::Arc;
use tracing::{debug, error, instrument, warn};

#[cfg(test)]
const BUF_SIZE: usize = 256 * 1024;
// 256 KB buffer, smaller for tests because they all run in parallel
#[cfg(not(test))]
const BUF_SIZE: usize = 1024 * 1024; // 1 MB buffer

#[instrument(skip(r, len), fields(len = len.to_formatted_string( & Locale::en)))]
pub fn seek_forward_exact(r: &mut impl Read, len: u64) -> io::Result<()> {
    debug!("");
    seek_forward(r, len, false)?;
    Ok(())
}

#[instrument(skip(r, len), fields(len = len.to_formatted_string( & Locale::en)))]
pub fn seek_forward<R: Read>(r: &mut R, len: u64, stop_on_eof: bool) -> io::Result<u64> {
    debug!("");
    if len == 0 {
        return Ok(0);
    }
    let mut buffer = vec![0; BUF_SIZE];
    let mut pos = 0_u64;
    loop {
        #[allow(clippy::cast_possible_truncation)]
        let bytes_to_read = if pos + buffer.len() as u64 > len {
            (len - pos) as usize
        } else {
            buffer.len()
        };
        if bytes_to_read == 0 {
            break;
        }
        let bytes_read = r.read(&mut buffer[..bytes_to_read]).inspect_err(|err| {
            error!(
                "error reading from file pos {} len {} {err}",
                pos.to_formatted_string(&Locale::en),
                bytes_to_read.to_formatted_string(&Locale::en)
            );
        })?;
        pos += bytes_read as u64;
        if pos == len {
            break;
        } else if bytes_read == 0 {
            if stop_on_eof {
                break;
            }
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "unexpected eof",
            ));
        }
    }

    Ok(pos)
}

#[instrument(skip(r, w, len), fields(len = len.to_formatted_string(& Locale::en)))]
pub fn copy_exact(r: &mut impl Read, w: &mut impl Write, len: u64) -> io::Result<()> {
    debug!("");
    copy(r, w, len, false)?;
    Ok(())
}

#[instrument(skip(r, w, len), fields(len = len.to_formatted_string(& Locale::en)))]
pub fn copy(r: &mut impl Read, w: &mut impl Write, len: u64, stop_on_eof: bool) -> io::Result<u64> {
    debug!("");
    if len == 0 {
        return Ok(0);
    }
    let mut buffer = vec![0; BUF_SIZE];
    let mut read_pos = 0_u64;
    loop {
        #[allow(clippy::cast_possible_truncation)]
        let buf_len = min(buffer.len(), (len - read_pos) as usize);
        let read = r.read(&mut buffer[..buf_len]).inspect_err(|err| {
            error!(
                "error reading from file pos {} len {} {err}",
                read_pos.to_formatted_string(&Locale::en),
                buf_len.to_formatted_string(&Locale::en)
            );
        })?;
        w.write_all(&buffer[..read]).inspect_err(|err| {
            error!(
                "error writing to file pos {} len {} {err}",
                read_pos.to_formatted_string(&Locale::en),
                buf_len.to_formatted_string(&Locale::en)
            );
        })?;
        read_pos += read as u64;
        if read_pos == len {
            break;
        }

        if read == 0 {
            if stop_on_eof {
                break;
            }
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "unexpected eof",
            ));
        }
    }
    Ok(read_pos)
}

#[instrument(skip(w, len), fields(len = len.to_formatted_string(& Locale::en)))]
pub fn fill_zeros(w: &mut impl Write, len: u64) -> io::Result<()> {
    debug!("");
    if len == 0 {
        return Ok(());
    }
    let buffer = vec![0; BUF_SIZE];
    let mut written = 0_u64;
    loop {
        #[allow(clippy::cast_possible_truncation)]
        let buf_len = min(buffer.len(), (len - written) as usize);
        w.write_all(&buffer[..buf_len]).inspect_err(|err| {
            error!(
                "error writing to file pos {} len {} {err}",
                written.to_formatted_string(&Locale::en),
                buf_len.to_formatted_string(&Locale::en)
            );
        })?;
        written += buf_len as u64;
        if written == len {
            break;
        }
    }
    Ok(())
}
/// Read trying to fill the buffer but stops on eof
#[instrument(skip(r, buf))]
pub fn read(mut r: impl Read, buf: &mut [u8]) -> io::Result<usize> {
    let mut bytes_read = 0;
    debug!("trying to fill the requested fuse buffer size {}",buf.len());
    loop {
        let len = r.read(&mut buf[bytes_read..])?;
        if len == 0 {
            break;
        }
        bytes_read += len; 
        // if we read less then buf.len (that has been requested by fuse)
        // we try to read once again however this might happen either because 
        // 1. there were not enough bytes in the buffer yet
        // 2. we read what was there and reached EOF -> BUG
        if bytes_read == buf.len() {
            break;
        }
    }
    debug!("managed to read total {}",bytes_read);
    Ok(bytes_read)
}

#[allow(dead_code)]
pub struct RandomReader {
    buf: Arc<Vec<u8>>,
    pos: usize,
}

impl RandomReader {
    #[allow(dead_code)]
    pub fn new(len: usize) -> Self {
        let mut buf = vec![0; len];
        rand::thread_rng().fill_bytes(&mut buf);
        Self {
            buf: Arc::new(buf),
            pos: 0,
        }
    }
}

impl Read for RandomReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pos > self.buf.len() {
            return Ok(0);
        }
        let len = buf.len().min(self.buf.len() - self.pos);
        buf[0..len].copy_from_slice(&self.buf[self.pos..self.pos + len]);
        self.pos += len;
        Ok(len)
    }
}

impl Seek for RandomReader {
    #[allow(clippy::cast_possible_wrap)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(pos) => pos as i64,
            SeekFrom::End(pos) => self.buf.len() as i64 + pos,
            SeekFrom::Current(pos) => self.pos as i64 + pos,
        };
        if new_pos < 0 || new_pos > self.buf.len() as i64 {
            return Err(Error::new(io::ErrorKind::InvalidInput, "outside of bounds"));
        }
        self.pos = new_pos as usize;
        Ok(new_pos as u64)
    }
}

impl Clone for RandomReader {
    fn clone(&self) -> Self {
        Self {
            buf: self.buf.clone(),
            pos: 0,
        }
    }
}
