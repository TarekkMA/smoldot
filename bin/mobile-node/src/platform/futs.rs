use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::future::BoxFuture;
use smoldot_light_base::ConnectError;
use crate::platform::connection::{Connection, Stream};

pub type Delay = BoxFuture<'static, ()>;

pub type ConnectFuture = BoxFuture<
    'static,
    Result<
        smoldot_light_base::PlatformConnection<Stream, Connection>,
        ConnectError,
    >,
>;

pub type StreamDataFuture = BoxFuture<'static, ()>;

pub type NextSubstreamFuture = BoxFuture<
    'static,
    Option<(Stream, smoldot_light_base::PlatformSubstreamDirection)>,
>;