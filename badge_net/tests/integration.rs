use badge_net::{
    read_frame, read_framed_value, write_frame, AsyncRead, AsyncWrite, Request, Update,
};

struct VecWrap(Vec<u8>);

impl AsyncWrite for VecWrap {
    type Error = &'static str;

    async fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.0.extend_from_slice(buf);
        Ok(())
    }
}

impl AsyncRead for VecWrap {
    type Error = &'static str;

    async fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let len = buf.len();
        buf.copy_from_slice(&self.0.get(..len).ok_or("not enough data")?);
        self.0 = self.0[len..].to_vec();
        Ok(())
    }
}

/// Full round trip testing with framing and serialization.  So nice!
#[tokio::test]
async fn test_framing() {
    let request = Request::Close;
    let mut stream = VecWrap(Vec::new());
    let mut buf = [0u8; 64];

    write_frame(&mut stream, &request, buf.as_mut_slice())
        .await
        .expect("pass");

    assert!(stream.0.len() > 0);

    let buf_frame = read_frame(&mut stream, buf.as_mut_slice())
        .await
        .expect("pass");

    let req2 = Request::try_from(buf_frame).expect("pass");

    assert_eq!(request, req2);
    assert_eq!(stream.0.len(), 0);

    let update = Update {
        text: "Hello World",
        freq: 123,
    };

    write_frame(&mut stream, &update, buf.as_mut_slice())
        .await
        .expect("pass");

    assert!(stream.0.len() > 0);

    let update2 = read_framed_value::<Update>(&mut stream, buf.as_mut_slice())
        .await
        .expect("pass");

    assert_eq!(update, update2);
    assert_eq!(stream.0.len(), 0);
}
