use std::future::Future;

use bytes::{Buf, BufMut};
use futures::{SinkExt, StreamExt};
use prost::Message;
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Encoder, Framed};

use crate::remote::{
    proto::Envelope,
    transport::{Connection, Transport, TransportError},
};

///Length prefixed codec for envelope messages over TCP
/// format : [4 bytes big-endian length][protobuf payload]
pub struct EnvelopeCodec;

impl Decoder for EnvelopeCodec {
    type Item = Envelope;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        //need at least 4 bytes for length prefix
        if src.len() < 4 {
            return Ok(None);
        }
        let len = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;

        if src.len() < 4 + len {
            //not enough data yet
            src.reserve(4 + len - src.len());
            return Ok(None);
        }

        src.advance(4); //consume length prefix

        let payload = src.split_to(len);

        let envelope = Envelope::decode(payload.as_ref())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(Some(envelope))
    }
}

impl Encoder<Envelope> for EnvelopeCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Envelope, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        let payload = item.to_bytes();
        let len = payload.len() as u32;

        dst.reserve(4 + payload.len());
        dst.put_u32(len);
        dst.extend_from_slice(&payload);
        Ok(())
    }
}

///TCP connection wrapper
pub struct TcpConnection {
    framed: Framed<TcpStream, EnvelopeCodec>,
    local_addr: String,
}

impl TcpConnection {
    pub fn new(stream: TcpStream) -> Self {
        let local_addr = stream
            .local_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        let framed = Framed::new(stream, EnvelopeCodec);
        TcpConnection { framed, local_addr }
    }

    /// Get the local socket address as a string
    pub fn local_addr(&self) -> &str {
        &self.local_addr
    }
}

impl Connection for TcpConnection {
    fn send(
        &mut self,
        envelope: Envelope,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), TransportError>> + Send + '_>> {
        Box::pin(async move {
            self.framed.send(envelope).await?;
            Ok(())
        })
    }

    fn recv(
        &mut self,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Envelope, TransportError>> + Send + '_>> {
        Box::pin(async move {
            match self.framed.next().await {
                Some(Ok(envelope)) => Ok(envelope),
                Some(Err(e)) => Err(TransportError::Io(e)),
                None => Err(TransportError::Disconnected),
            }
        })
    }

    fn close(
        &mut self,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<(), TransportError>> + Send + '_>> {
        Box::pin(async move {
            self.framed.close().await?;
            Ok(())
        })
    }
}

pub struct TcpTransport;

impl Transport for TcpTransport {
    type Conn = TcpConnection;

    fn connect(
        &self,
        addr: &str,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Self::Conn, TransportError>> + Send + '_>>
    {
        let addr = addr.to_string();
        Box::pin(async move {
            let stream = TcpStream::connect(addr).await?;
            Ok(TcpConnection::new(stream))
        })
    }
}
