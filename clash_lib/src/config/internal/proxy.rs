use crate::config::utils;
use crate::Error;
use serde::de::value::MapDeserializer;
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

pub const PROXY_DIRECT: &str = "DIRECT";
pub const PROXY_REJECT: &str = "REJECT";
pub const PROXY_GLOBAL: &str = "GLOBAL";

pub enum OutboundProxy {
    ProxyServer(OutboundProxyProtocol),
    ProxyGroup(OutboundGroupProtocol),
}

impl OutboundProxy {
    pub(crate) fn name(&self) -> String {
        match self {
            OutboundProxy::ProxyServer(s) => s.name().to_string(),
            OutboundProxy::ProxyGroup(g) => g.name().to_string(),
        }
    }
}

fn map_serde_error(x: serde_yaml::Error) -> crate::Error {
    Error::InvalidConfig(if let Some(loc) = x.location() {
        format!(
            "{}, line, {}, column: {}",
            x.to_string(),
            loc.line(),
            loc.column()
        )
    } else {
        x.to_string()
    })
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum OutboundProxyProtocol {
    #[serde(skip)]
    Direct,
    #[serde(skip)]
    Reject,
    #[serde(rename = "ss")]
    Ss(OutboundShadowsocks),
    #[serde(rename = "socks5")]
    Socks5(OutboundSocks5),
    #[serde(rename = "trojan")]
    Trojan(OutboundTrojan),
}

impl OutboundProxyProtocol {
    fn name(&self) -> &str {
        match &self {
            OutboundProxyProtocol::Direct => PROXY_DIRECT,
            OutboundProxyProtocol::Reject => PROXY_REJECT,
            OutboundProxyProtocol::Ss(ss) => &ss.name,
            OutboundProxyProtocol::Socks5(socks5) => &socks5.name,
            OutboundProxyProtocol::Trojan(trojan) => &trojan.name,
        }
    }
}

impl TryFrom<HashMap<String, Value>> for OutboundProxyProtocol {
    type Error = crate::Error;

    fn try_from(mapping: HashMap<String, Value>) -> Result<Self, Self::Error> {
        OutboundProxyProtocol::deserialize(MapDeserializer::new(mapping.into_iter()))
            .map_err(map_serde_error)
    }
}

impl Display for OutboundProxyProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutboundProxyProtocol::Ss(_) => write!(f, "Shadowsocks"),
            OutboundProxyProtocol::Socks5(_) => write!(f, "Socks5"),
            OutboundProxyProtocol::Direct => write!(f, "{}", PROXY_DIRECT),
            OutboundProxyProtocol::Reject => write!(f, "{}", PROXY_REJECT),
            OutboundProxyProtocol::Trojan(_) => write!(f, "{}", "Trojan"),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct OutboundShadowsocks {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub cipher: String,
    pub password: String,
    pub udp: bool,
    pub plugin: Option<String>,
    pub plugin_opts: Option<HashMap<String, serde_yaml::Value>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct OutboundSocks5 {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub tls: bool,
    pub skip_cert_verity: bool,
    pub udp: bool,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct WsOpt {
    pub path: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub max_early_data: Option<i32>,
    pub early_data_header_name: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct GrpcOpt {
    pub grpc_service_name: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct OutboundTrojan {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub password: String,
    pub alpn: Option<Vec<String>>,
    pub sni: Option<String>,
    pub skip_cert_verify: Option<bool>,
    pub udp: Option<bool>,
    pub network: Option<String>,
    pub grpc_opts: Option<GrpcOpt>,
    pub ws_opts: Option<WsOpt>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(tag = "type")]
pub enum OutboundGroupProtocol {
    #[serde(rename = "relay")]
    Relay(OutboundGroupRelay),
    #[serde(rename = "url-test")]
    UrlTest(OutboundGroupUrlTest),
    #[serde(rename = "fallback")]
    Fallback(OutboundGroupFallback),
    #[serde(rename = "load-balance")]
    LoadBalance(OutboundGroupLoadBalance),
    #[serde(rename = "select")]
    Select(OutboundGroupSelect),
}

impl OutboundGroupProtocol {
    fn name(&self) -> &str {
        match &self {
            OutboundGroupProtocol::Relay(g) => &g.name,
            OutboundGroupProtocol::UrlTest(g) => &g.name,
            OutboundGroupProtocol::Fallback(g) => &g.name,
            OutboundGroupProtocol::LoadBalance(g) => &g.name,
            OutboundGroupProtocol::Select(g) => &g.name,
        }
    }
}

impl TryFrom<HashMap<String, Value>> for OutboundGroupProtocol {
    type Error = Error;

    fn try_from(mapping: HashMap<String, Value>) -> Result<Self, Self::Error> {
        OutboundGroupProtocol::deserialize(MapDeserializer::new(mapping.into_iter()))
            .map_err(map_serde_error)
    }
}

impl Display for OutboundGroupProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OutboundGroupProtocol::Relay(g) => write!(f, "{}", g.name),
            OutboundGroupProtocol::UrlTest(g) => write!(f, "{}", g.name),
            OutboundGroupProtocol::Fallback(g) => write!(f, "{}", g.name),
            OutboundGroupProtocol::LoadBalance(g) => write!(f, "{}", g.name),
            OutboundGroupProtocol::Select(g) => write!(f, "{}", g.name),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct OutboundGroupRelay {
    pub name: String,
    pub proxies: Option<Vec<String>>,
    #[serde(rename = "use")]
    pub use_provider: Option<Vec<String>>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct OutboundGroupUrlTest {
    pub name: String,

    pub proxies: Option<Vec<String>>,
    #[serde(rename = "use")]
    pub use_provider: Option<Vec<String>>,

    pub url: String,
    #[serde(deserialize_with = "utils::deserialize_u64")]
    pub interval: u64,
    pub tolerance: Option<i32>,
    pub lazy: Option<bool>,
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]

pub struct OutboundGroupFallback {
    pub name: String,

    pub proxies: Vec<String>,
    pub url: String,
    #[serde(deserialize_with = "utils::deserialize_u64")]
    pub interval: u64,
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]

pub struct OutboundGroupLoadBalance {
    pub name: String,

    pub proxies: Option<Vec<String>>,
    #[serde(rename = "use")]
    pub use_provider: Option<Vec<String>>,

    pub url: String,
    #[serde(deserialize_with = "utils::deserialize_u64")]
    pub interval: u64,
    pub strategy: Option<LoadBalanceStrategy>,
}
#[derive(serde::Serialize, serde::Deserialize, Debug)]

pub enum LoadBalanceStrategy {
    #[serde(rename = "consistent-hashing")]
    ConsistentHashing,
    #[serde(rename = "round-robin")]
    RoundRobin,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct OutboundGroupSelect {
    pub name: String,

    pub proxies: Option<Vec<String>>,
    #[serde(rename = "use")]
    pub use_provider: Option<Vec<String>>,
}