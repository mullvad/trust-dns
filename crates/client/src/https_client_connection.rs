// Copyright 2015-2018 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! UDP based DNS client connection for Client impls

use std::net::SocketAddr;
use std::sync::Arc;

use rustls::{Certificate, ClientConfig};
use trust_dns_proto::https::{HttpsClientConnect, HttpsClientStream, HttpsClientStreamBuilder};
use trust_dns_proto::tcp::TcpConnector;

use crate::client::{ClientConnection, Signer};

/// UDP based DNS Client connection
///
/// Use with `trust_dns_client::client::Client` impls
#[derive(Clone)]
pub struct HttpsClientConnection<T> {
    name_server: SocketAddr,
    dns_name: String,
    client_config: ClientConfig,
    connector: T,
}

impl<T: TcpConnector + Default> HttpsClientConnection<T> {
    /// Creates a new client connection with a default TCP connector.
    ///
    /// *Note* this has side affects of binding the socket to 0.0.0.0 and starting the listening
    ///        event_loop. Expect this to change in the future.
    ///
    /// # Arguments
    ///
    /// * `name_server` - address of the name server to use for queries
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> HttpsClientConnectionBuilder<T> {
        HttpsClientConnectionBuilder::default()
    }
}

impl<T: TcpConnector> HttpsClientConnection<T> {
    /// Creates a new client connection with a TCP connector.
    ///
    /// *Note* this has side affects of binding the socket to 0.0.0.0 and starting the listening
    ///        event_loop. Expect this to change in the future.
    ///
    /// # Arguments
    ///
    /// * `connector` - TCP connector to be used for establishing an HTTPS connection.
    pub fn with_connector(connector: T) -> HttpsClientConnectionBuilder<T> {
        HttpsClientConnectionBuilder::new(connector)
    }
}

impl<T> ClientConnection for HttpsClientConnection<T>
where
    T: TcpConnector,
{
    type Sender = HttpsClientStream;
    type SenderFuture = HttpsClientConnect<T>;

    fn new_stream(
        &self,
        // TODO: maybe signer needs to be applied in https...
        _signer: Option<Arc<Signer>>,
    ) -> Self::SenderFuture {
        // TODO: maybe signer needs to be applied in https...
        let https_builder = HttpsClientStreamBuilder::with_client_config(
            self.connector.clone(),
            Arc::new(self.client_config.clone()),
        );
        https_builder.build(self.name_server, self.dns_name.clone())
    }
}

/// A helper to construct an HTTPS connection
pub struct HttpsClientConnectionBuilder<T: TcpConnector> {
    client_config: ClientConfig,
    connector: T,
}

impl<T: TcpConnector> HttpsClientConnectionBuilder<T> {
    /// Return a new builder for DNS-over-HTTPS
    pub fn new(connector: T) -> HttpsClientConnectionBuilder<T> {
        HttpsClientConnectionBuilder {
            client_config: ClientConfig::new(),
            connector,
        }
    }

    /// Constructs a new TlsStreamBuilder with the associated ClientConfig
    pub fn with_client_config(client_config: ClientConfig, connector: T) -> Self {
        HttpsClientConnectionBuilder {
            client_config,
            connector,
        }
    }

    /// Add a custom trusted peer certificate or certificate authority.
    ///
    /// If this is the 'client' then the 'server' must have it associated as it's `identity`, or have had the `identity` signed by this certificate.
    pub fn add_ca(&mut self, ca: Certificate) {
        self.client_config
            .root_store
            .add(&ca)
            .expect("bad certificate!");
    }

    /// Creates a new HttpsStream to the specified name_server
    ///
    /// # Arguments
    ///
    /// * `name_server` - IP and Port for the remote DNS resolver
    /// * `dns_name` - The DNS name, Subject Public Key Info (SPKI) name, as associated to a certificate
    pub fn build(self, name_server: SocketAddr, dns_name: String) -> HttpsClientConnection<T> {
        HttpsClientConnection {
            name_server,
            dns_name,
            client_config: self.client_config,
            connector: self.connector,
        }
    }
}

impl<T: TcpConnector + Default> Default for HttpsClientConnectionBuilder<T> {
    fn default() -> Self {
        Self {
            client_config: ClientConfig::new(),
            connector: Default::default(),
        }
    }
}
