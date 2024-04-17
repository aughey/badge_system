#![no_std]

use serde::{Deserialize, Serialize};

/// Request from the badge to the server
#[derive(Debug, Serialize, Deserialize)]
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

/// Response from the server to the badge
#[derive(Debug, Serialize, Deserialize)]
pub struct Update<'a> {
    /// Text to display on the badge
    pub text: &'a str,
    /// Frequency of the LED
    pub freq: u8,
}
impl Update<'_> {
    /// Serialize the update
    pub fn serialize<'a>(&self, buf: &'a mut [u8; 64]) -> Result<&'a [u8], postcard::Error> {
        Ok(postcard::to_slice(self, buf)?)
    }
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
