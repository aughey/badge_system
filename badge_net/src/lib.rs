#![no_std]

use serde::{Deserialize, Serialize};

/// Request from the badge to the server
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Request {
    /// Ready for new data
    Ready,
    /// Close the connection
    Close,
}
impl Request {
    /// Serialize the request
    pub fn serialize(&self) -> [u8; 1] {
        let mut buf = [0u8; 1];
        postcard::to_slice(self, &mut buf).unwrap();
        buf
    }
}
impl TryFrom<&[u8]> for Request {
    type Error = postcard::Error;

    fn try_from(value: &[u8]) -> Result<Request, Self::Error> {
        postcard::from_bytes(value)
    }
}

/// Response from the server to the device
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Update<'a> {
    /// Text to display
    pub text: &'a str,
    /// Frequency of the LED
    pub freq: u8,
}
impl Update<'_> {
    /// Serialize the update
    pub fn serialize<'a>(&self, buf: &'a mut [u8]) -> Result<&'a [u8], postcard::Error> {
        Ok(postcard::to_slice(self, buf)?)
    }
}
impl<'a> TryFrom<&'a [u8]> for Update<'a> {
    type Error = postcard::Error;

    fn try_from(value: &[u8]) -> Result<Update, Self::Error> {
        postcard::from_bytes(value)
    }
}

/// Unfortunately we need our own trait here because the traits between `embedded-io-async` and
/// `tokio` aren't the same, so we define our framing and data sending traits here.
#[allow(async_fn_in_trait)]
pub trait AsyncRead {
    type Error;
    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error>;
}

/// Unfortunately we need our own trait here because the traits between `embedded-io-async` and
/// `tokio` aren't the same, so we define our framing and data sending traits here.
#[allow(async_fn_in_trait)]
pub trait AsyncWrite {
    type Error;
    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error>;
}

/// Reads a frame from the stream.  
/// This frame is untyped meaning that we are just going to read a len + data value out
/// of the stream and provide the raw buffer.  Errors on len not fitting buffer or read errors.
/// This is not cancel-safe.
pub async fn read_frame<'a>(
    stream: &mut impl AsyncRead,
    buf: &'a mut [u8],
) -> Result<&'a [u8], &'static str> {
    // buf is framed by a u32 len and then the data

    // use the provided buf to read the len
    stream
        .read_exact(
            buf.get_mut(..4)
                .ok_or_else(|| "supplied buf not big enough for len")?,
        )
        .await
        .map_err(|_| "Failed to read len")?;
    let len: usize = u32::from_le_bytes(
        buf.get(..4)
            .expect("already tested buf len above")
            .try_into()
            .expect("u32 must be 4 bytes"),
    )
    .try_into()
    .map_err(|_| "Failed to convert len to usize")?;

    // use the provided buf to read the data
    stream
        .read_exact(
            buf.get_mut(..len as usize)
                .ok_or_else(|| "supplied buf not big enough for data")?,
        )
        .await
        .map_err(|_| "Failed to read data")?;

    Ok(buf.get(..len).expect("already tested buf len above"))
}

/// Read a deserializable value from the stream using the given buffer as scratch space.
/// The buffer must be at least the size of the deserialized value.
/// This is not cancel-safe.
pub async fn read_framed_value<'a, T>(
    stream: &mut impl AsyncRead,
    buf: &'a mut [u8],
) -> Result<T, &'static str>
where
    T: Deserialize<'a>,
{
    let buf = read_frame(stream, buf).await?;
    postcard::from_bytes(buf).map_err(|_| "Failed to deserialize data")
}

/// Writes a serializable value to the stream framing it with len + data.
/// This is not cancel-safe.
pub async fn write_frame<T>(
    stream: &mut impl AsyncWrite,
    value: &T,
    buf: &mut [u8],
) -> Result<(), &'static str>
where
    T: Serialize,
{
    // buf is framed by a u32 len and then the data

    // use the provided buf to write the data
    let buf = postcard::to_slice(value, buf).map_err(|_| "Failed to serialize data")?;
    let len: u32 = buf
        .len()
        .try_into()
        .map_err(|_| "Failed to convert len to u32")?;

    stream
        .write_all(&len.to_le_bytes())
        .await
        .map_err(|_| "Failed to write len")?;
    stream
        .write_all(buf)
        .await
        .map_err(|_| "Failed to write data")?;

    Ok(())
}

#[test]
fn test_request_serialize() {
    let mut buf = [0u8; 64];
    let msg = Request::Ready;
    let buf = postcard::to_slice(&msg, &mut buf).unwrap();
    assert!(buf.len() > 0);
    assert_eq!(buf[0], 0);

    let req: Request = postcard::from_bytes(buf).unwrap();
    assert!(matches!(req, Request::Ready));

    let mut buf2 = [0u8; 64];
    let msg = Request::Close;
    let buf2 = postcard::to_slice(&msg, &mut buf2).unwrap();
    assert!(buf2.len() > 0);
    assert_ne!(buf, buf2);

    let req2: Request = postcard::from_bytes(buf2).unwrap();
    assert!(matches!(req2, Request::Close));
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_update_serialize() {
        let mut buf = [0u8; 64];
        let msg = Update {
            text: "Hello, World!",
            freq: 10,
        };
        let buf = postcard::to_slice(&msg, &mut buf).unwrap();
        assert!(buf.len() > 0);

        let update: Update = postcard::from_bytes(buf).unwrap();
        assert_eq!(update.text, "Hello, World!");
        assert_eq!(update.freq, 10);
    }

    #[test]
    fn test_self_serialize() {
        let req = Request::Ready;
        let buf = req.serialize();
        assert!(buf.len() > 0);
        let req2 = Request::try_from(buf.as_slice()).unwrap();
        assert_eq!(req, req2);

        let update = Update {
            text: "Hello, World!",
            freq: 10,
        };
        let mut buf = [0u8; 64];
        let buf = update.serialize(&mut buf).unwrap();
        assert!(buf.len() > 0);
        let update2 = Update::try_from(buf).unwrap();
        assert_eq!(update, update2);

        // too small buf will fail serialize
        let mut buf = [0u8; 1];
        let buf = update.serialize(&mut buf);
        assert!(buf.is_err());
    }
}
