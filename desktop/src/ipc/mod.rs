use std::io::{self, Read, Write};
use std::marker::PhantomData;


use interprocess::local_socket::{ConnectOptions, GenericNamespaced, ListenerOptions, Stream, ToNsName};
use interprocess::local_socket::tokio::{Listener as TokioListener, Stream as TokioStream};
use interprocess::local_socket::traits::tokio::Listener as TokioListenerTrait;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::APP_NAME;

pub trait IPCServerExt: Sized {
    fn create_server() -> Self;
    async fn recieve(&mut self) -> ServerBound;
}

impl IPCServerExt for TokioListener {
    fn create_server() -> Self {
        let socket_name = APP_NAME
            .to_ns_name::<GenericNamespaced>()
            .unwrap();

        ListenerOptions::new()
            .name(socket_name)
            .create_tokio()
            .unwrap()
    }
    async fn recieve(&mut self) -> ServerBound {
        let mut stream = self
            .accept()
            .await
            .unwrap();

        let mut buf = [0u8];
        stream.read_exact(&mut buf)
            .await
            .unwrap();

        match buf[0] {
            0x00..=0x0F => {
                let signal = Signal::try_from(buf[0]).unwrap();

                ServerBound::Signal(signal)
            },
            0x10..=0x1F => {
                let request = Request::try_from(buf[0]).unwrap();
                let response_token = ResponseToken::create(stream);

                ServerBound::Request(request, response_token)
            },
            _ => panic!("Unknown byte"),
        }
    }
}

pub trait IPCClientExt: Sized {
    fn create_client() -> io::Result<Self>;
    fn send_signal(self, signal: Signal);
    fn send_request(self, request: Request) -> Response;
}

impl IPCClientExt for Stream {
    fn create_client() -> io::Result<Self> {
        let socket_name = APP_NAME
            .to_ns_name::<GenericNamespaced>()
            .unwrap();

        let stream_result = ConnectOptions::new()
            .name(socket_name)
            .connect_sync();

        stream_result
    }
    fn send_signal(mut self, signal: Signal) {
        self.write_all(&[signal as u8]).unwrap();
    }
    fn send_request(mut self, request: Request) -> Response {
        self.write_all(&[request as u8]).unwrap();

        let mut buf = [0u8];

        self.read_exact(&mut buf).unwrap();

        buf[0].try_into().unwrap()
    }
}

/// The server recieves ServerBound as a message from a client
/// It should use the TryFrom<u8> for ServerBound when accepting a message
#[repr(u8)]
pub enum ServerBound {
    Signal(Signal),
    Request(Request, ResponseToken),
}


/// Discriminant values must be in the range 0x00..=0x0F.
/// Other values are reserved for other message enums.
#[repr(u8)]
pub enum Signal {
    GUIStarted = 0x00,
    GUIExited = 0x01,
}


impl TryFrom<u8> for Signal {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Signal::GUIStarted),
            0x01 => Ok(Signal::GUIExited),
            _ => Err(())
        }
    }
}

/// Discriminant values must be in the range 0x10..=0x1F.
/// Other values are reserved for other message enums.

#[repr(u8)]
pub enum Request {
    CanUninstall = 0x10
}

impl TryFrom<u8> for Request {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(Request::CanUninstall),
            _ => Err(())
        }
    }
}


/// Clients recieves Response as a message from the server 
/// It should use the TryFrom<u8> for Response when accepting a message
#[repr(u8)]
pub enum Response {
    CanUninstall = 0,
    CanNotUninstall = 1
}

impl TryFrom<u8> for Response {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Response::CanUninstall),
            1 => Ok(Response::CanNotUninstall),
            _ => Err(())
        }
    }
}

pub struct ResponseToken {
    stream: TokioStream,
    responded: bool
}

impl ResponseToken {
    fn create(stream: TokioStream) -> Self {
        Self {
            stream,
            responded: false
        }
    }
    pub async fn respond(mut self, response: Response) {
        self.stream
            .write_all(&[response as u8])
            .await
            .unwrap();

        self.responded = true;
    }
}

impl Drop for ResponseToken {
    fn drop(&mut self) {
        if !self.responded {
            panic!("ResponseToken dropped without sending a response");
        }
    }
}

