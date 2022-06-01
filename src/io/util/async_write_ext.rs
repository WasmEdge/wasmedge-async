use crate::io::util::flush::{flush, Flush};
use crate::io::util::shutdown::{shutdown, Shutdown};
use crate::io::util::write::{write, Write};
use crate::io::AsyncWrite;
/// Writes bytes to a sink.
///
/// Implemented as an extension trait, adding utility methods to all
/// [`AsyncWrite`] types. Callers will tend to import this trait instead of
/// [`AsyncWrite`].
///
/// See [module][crate::io] documentation for more details.
///
/// [`AsyncWrite`]: AsyncWrite
pub trait AsyncWriteExt: AsyncWrite {
    /// Writes a buffer into this writer, returning how many bytes were
    /// written.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
    /// ```
    ///
    /// This function will attempt to write the entire contents of `buf`, but
    /// the entire write may not succeed, or the write may also generate an
    /// error. A call to `write` represents *at most one* attempt to write to
    /// any wrapped object.
    ///
    /// # Return
    ///
    /// If the return value is `Ok(n)` then it must be guaranteed that `n <=
    /// buf.len()`. A return value of `0` typically means that the
    /// underlying object is no longer able to accept bytes and will likely
    /// not be able to in the future as well, or that the buffer provided is
    /// empty.
    ///
    /// # Errors
    ///
    /// Each call to `write` may generate an I/O error indicating that the
    /// operation could not be completed. If an error is returned then no bytes
    /// in the buffer were written to this writer.
    ///
    /// It is **not** considered an error if the entire buffer could not be
    /// written to this writer.
    fn write<'a>(&'a mut self, src: &'a [u8]) -> Write<'a, Self>
    where
        Self: Unpin,
    {
        write(self, src)
    }

    /// Flushes this output stream, ensuring that all intermediately buffered
    /// contents reach their destination.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn flush(&mut self) -> io::Result<()>;
    /// ```
    ///
    /// # Errors
    ///
    /// It is considered an error if not all bytes could be written due to
    /// I/O errors or EOF being reached.
    ///
    fn flush(&mut self) -> Flush<'_, Self>
    where
        Self: Unpin,
    {
        flush(self)
    }

    /// Shuts down the output stream, ensuring that the value can be dropped
    /// cleanly.
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn shutdown(&mut self) -> io::Result<()>;
    /// ```
    ///
    /// Similar to [`flush`], all intermediately buffered is written to the
    /// underlying stream. Once the operation completes, the caller should
    /// no longer attempt to write to the stream. For example, the
    /// `TcpStream` implementation will issue a `shutdown(Write)` sys call.
    ///
    /// [`flush`]: fn@crate::io::AsyncWriteExt::flush
    fn shutdown(&mut self) -> Shutdown<'_, Self>
    where
        Self: Unpin,
    {
        shutdown(self)
    }
}

impl<W: AsyncWrite + ?Sized> AsyncWriteExt for W {}
