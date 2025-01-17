/// Chain configuration settings.
#[allow(dead_code)]
pub struct ChainConfig {
    /// RPC endpoint URL for the chain
    rpc_url: String,

    /// WebSocket endpoint URL for real-time communication
    ws_url: String,

    /// GossipSub protocol endpoint URL for peer-to-peer messaging
    gossipsub_url: String,

    /// List of bootstrap peer addresses in multiaddr format
    peers: Vec<String>,

    /// Chain
    chain: Chain,
}

impl ChainConfig {
    pub fn rpc_url(&self) -> &String {
        &self.rpc_url
    }

    pub fn ws_url(&self) -> &String {
        &self.ws_url
    }

    pub fn gossipsub_url(&self) -> &String {
        &self.gossipsub_url
    }

    pub fn peers(&self) -> &Vec<String> {
        &self.peers
    }
    pub fn chain(&self) -> Chain {
        self.chain.clone()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Chain {
    Ethereum,
    Optimism,
    Base,
}

/// Builder for creating a Chain configuration.
#[allow(dead_code)]
pub struct ChainConfigBuilder {
    /// Optional RPC endpoint URL
    rpc_url: Option<String>,

    /// Optional WebSocket endpoint URL
    ws_url: Option<String>,

    /// Optional GossipSub protocol endpoint URL
    gossipsub_url: Option<String>,

    /// List of bootstrap peer addresses
    peers: Vec<String>,

    /// Chain
    chain: Option<Chain>,
}

#[allow(dead_code)]
impl ChainConfigBuilder {
    pub fn builder() -> ChainConfigBuilder {
        ChainConfigBuilder {
            gossipsub_url: None,
            ws_url: None,
            rpc_url: None,
            peers: vec![],
            chain: None,
        }
    }
    /// Sets the RPC endpoint URL for the chain configuration
    ///
    /// # Arguments
    /// * `rpc_url` - The RPC endpoint URL as a string
    pub fn rpc(&mut self, rpc_url: &str) -> &mut ChainConfigBuilder {
        self.rpc_url = Some(rpc_url.to_string());
        self
    }

    /// Sets the WebSocket endpoint URL for the chain configuration
    ///
    /// # Arguments
    /// * `ws_url` - The WebSocket endpoint URL as a string
    pub fn ws(&mut self, ws_url: &str) -> &mut ChainConfigBuilder {
        self.ws_url = Some(ws_url.to_string());
        self
    }

    /// Sets the GossipSub protocol endpoint URL for the chain configuration
    ///
    /// # Arguments
    /// * `gossipsub_url` - The GossipSub endpoint URL as a string
    pub fn gossipsub(&mut self, gossipsub_url: &str) -> &mut ChainConfigBuilder {
        self.gossipsub_url = Some(gossipsub_url.to_string());
        self
    }

    /// Sets the list of bootstrap peers for the chain configuration
    ///
    /// # Arguments
    /// * `peers` - Vector of peer addresses in multiaddr format
    pub fn bootstrap_peers(&mut self, peers: Vec<String>) -> &mut ChainConfigBuilder {
        self.peers = peers;
        self
    }

    /// Sets the chain for the chain configuration
    ///
    /// # Arguments
    /// * `peers` - Vector of peer addresses in multiaddr format
    pub fn chain(&mut self, chain: Chain) -> &mut ChainConfigBuilder {
        self.chain = Some(chain);
        self
    }

    /// Builds the final Chain configuration
    pub fn build(&mut self) -> ChainConfig {
        if self.gossipsub_url.is_none() && self.rpc_url.is_none() && self.ws_url.is_none() {
            panic!("at least one url is required.")
        }

        ChainConfig {
            rpc_url: self.rpc_url.clone().unwrap_or_default(),
            ws_url: self.ws_url.clone().unwrap_or_default(),
            gossipsub_url: self.gossipsub_url.clone().unwrap_or_default(),
            peers: self.peers.clone(),
            chain: self.chain.clone().expect("chain is required."),
        }
    }
}

#[allow(dead_code)]
impl Chain {
    // Creates a new ChainConfigBuilder instance to construct a Chain configuration
    ///
    /// # Example
    /// ```ignore
    /// let chain = Chain::builder()
    ///     .rpc("https://mainnet.example.io".to_string())
    ///     .ws("wss://...".to_string())
    ///     .gossipsub("/ip4/0.0.0.0/tcp/9000".to_string())
    ///     .bootstrap_peers(vec![
    ///          "/ip4/x.x.x.x/tcp/9000/p2p/QmPeer1...".to_string()
    ///      ]);
    /// ```
    pub fn builder() -> ChainConfigBuilder {
        ChainConfigBuilder {
            rpc_url: None,
            ws_url: None,
            gossipsub_url: None,
            peers: vec![],
            chain: None,
        }
    }
}
