mod instant;
mod futs;
mod connection;

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use futures::FutureExt;
use crate::platform::connection::{Connection, Stream};
use crate::platform::futs::{ConnectFuture, Delay, StreamDataFuture, NextSubstreamFuture};
use crate::platform::instant::Instant;

pub struct Platform;

impl smoldot_light_base::Platform for Platform {
    type Delay = Delay;
    type Instant = Instant;
    type Connection = Connection;
    type Stream = Stream;
    type ConnectFuture = ConnectFuture;
    type StreamDataFuture = StreamDataFuture;
    type NextSubstreamFuture = NextSubstreamFuture;

    fn now_from_unix_epoch() -> Duration {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
    }

    fn now() -> Self::Instant {
        Instant::now()
    }

    fn sleep(duration: Duration) -> Self::Delay {
        tokio::time::sleep(duration).boxed()
    }

    fn sleep_until(when: Self::Instant) -> Self::Delay {
        tokio::time::sleep(when - Instant::now()).boxed()
    }

    fn connect(url: &str) -> Self::ConnectFuture {
        todo!()
    }

    fn open_out_substream(connection: &mut Self::Connection) {
        todo!()
    }

    fn next_substream(connection: &mut Self::Connection) -> Self::NextSubstreamFuture {
        todo!()
    }

    fn wait_more_data(stream: &mut Self::Stream) -> Self::StreamDataFuture {
        todo!()
    }

    fn read_buffer(stream: &mut Self::Stream) -> Option<&[u8]> {
        todo!()
    }

    fn advance_read_cursor(stream: &mut Self::Stream, bytes: usize) {
        todo!()
    }

    fn send(stream: &mut Self::Stream, data: &[u8]) {
        todo!()
    }
}