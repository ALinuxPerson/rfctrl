use tokio::fs::File;
use std::{io, thread};
use std::pin::Pin;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::task::{Context, Poll};
use futures::{Stream, Future};
use tokio::io::AsyncReadExt;
use crate::{CHAR_DEVICE, Event};
use crate::base::ReadFrom;

pub struct Events {
    file: File,
    buffer: [u8; Event::SIZE],
}

impl Events {
    async fn next_event(&mut self) -> Option<io::Result<Event>> {
        let bytes_read = match self.file.read(&mut self.buffer).await {
            Ok(bytes_read) => bytes_read,
            Err(error) => return Some(Err(error)),
        };

        if bytes_read == 0 {
            return None
        }

        if bytes_read != 8 && bytes_read != 9 {
            return Some(Err(io::Error::new(io::ErrorKind::UnexpectedEof, format!("read {bytes_read} bytes but expected to read 8 or 9 bytes"))) )
        }

        Some(Event::read_from(&self.buffer[..bytes_read]))
    }
}

impl Stream for Events {
    type Item = io::Result<Event>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let next = self.next_event();
        tokio::pin!(next);

        next.poll(cx)
    }
}

pub async fn events() -> io::Result<Events> {
    Ok(Events {
        file: File::open(CHAR_DEVICE).await?,
        buffer: [0; Event::SIZE],
    })
}
