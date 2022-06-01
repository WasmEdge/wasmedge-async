use crate::io::util::read::{read, Read};
use crate::io::AsyncRead;

/// [`AsyncRead`]: AsyncRead
pub trait AsyncReadExt: AsyncRead {
    /// Pulls some bytes from this source into the specified buffer,
    /// returning how many bytes were read.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
    /// ```
    ///
    /// This method does not provide any guarantees about whether it
    /// completes immediately or asynchronously.
    ///
    /// # Return
    ///
    /// If the return value of this method is `Ok(n)`, then it must be
    /// guaranteed that `0 <= n <= buf.len()`. A nonzero `n` value indicates
    /// that the buffer `buf` has been filled in with `n` bytes of data from
    /// this source. If `n` is `0`, then it can indicate one of two
    /// scenarios:
    ///
    /// 1. This reader has reached its "end of file" and will likely no longer
    ///    be able to produce bytes. Note that this does not mean that the
    ///    reader will *always* no longer be able to produce bytes.
    /// 2. The buffer specified was 0 bytes in length.
    ///
    /// No guarantees are provided about the contents of `buf` when this
    /// function is called, implementations cannot rely on any property of the
    /// contents of `buf` being `true`. It is recommended that *implementations*
    /// only write data to `buf` instead of reading its contents.
    ///
    /// Correspondingly, however, *callers* of this method may not assume
    /// any guarantees about how the implementation uses `buf`. It is
    /// possible that the code that's supposed to write to the buffer might
    /// also read from it. It is your responsibility to make sure that `buf`
    /// is initialized before calling `read`.
    ///
    /// # Errors
    ///
    /// If this function encounters any form of I/O or other error, an error
    /// variant will be returned. If an error is returned then it must be
    /// guaranteed that no bytes were read.
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Read<'a, Self>
    where
        Self: Unpin,
    {
        read(self, buf)
    }
}

impl<R: AsyncRead + ?Sized> AsyncReadExt for R {}
