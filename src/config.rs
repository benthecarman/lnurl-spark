use bitcoin::Network;
use clap::Parser;
use spark_wallet::SparkWalletConfig;

#[derive(Parser, Debug, Clone)]
#[command(version, author, about)]
/// A simple LNURL pay server. Allows you to have a lightning address for your own node.
pub struct Config {
    /// Postgres connection string (e.g. postgres://user:password@localhost/dbname)
    #[clap(long, env = "LNURL_PG_URL")]
    pub pg_url: String,

    /// Nostr nsec used for zaps
    #[clap(long, env = "LNURL_NSEC")]
    pub nsec: String,

    /// Bind address for lnurl-server's webserver
    #[clap(default_value_t = String::from("0.0.0.0"), long, env = "LNURL_BIND")]
    pub bind: String,

    /// Port for lnurl-server's webserver
    #[clap(default_value_t = 3000, long, env = "LNURL_PORT")]
    pub port: u16,

    /// Network lnd is running on ["bitcoin", "testnet", "signet, "regtest"]
    #[clap(default_value_t = Network::Bitcoin, short, long, env = "LNURL_NETWORK")]
    pub network: Network,

    /// Minimum amount in millisatoshis that can be sent via LNURL
    #[clap(default_value_t = 1_000, long, env = "LNURL_MIN_SENDABLE")]
    pub min_sendable: u64,

    /// Maximum amount in millisatoshis that can be sent via LNURL
    #[clap(default_value_t = 11_000_000_000, long, env = "LNURL_MAX_SENDABLE")]
    pub max_sendable: u64,

    /// The domain name you are running lnurl-server on
    #[clap(default_value_t = String::from("localhost:3000"), long, env = "LNURL_DOMAIN")]
    pub domain: String,
}

impl Config {
    pub fn spark_config(&self) -> SparkWalletConfig {
        SparkWalletConfig::default_config(self.network.try_into().expect("Invalid network"))
    }
}
