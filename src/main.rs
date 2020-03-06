mod config;

use clap::{App, Arg};
use cloudflare::endpoints::dns;
use cloudflare::framework::async_api::{ApiClient, Client};
use cloudflare::framework::auth::Credentials;
use cloudflare::framework::{Environment, HttpApiClientConfig};
use log::{debug, info};
use std::net::Ipv4Addr;

// The data structure that the DNS query produces is not in the right shape
// for the DNS update. Thus we project the query into a SimpleDnsRecord and
// construct the update using it.
struct SimpleDnsRecord {
    name: String,
    id: String,
    ttl: u32,
    proxied: bool,
    ip: Ipv4Addr,
}

async fn get_current_ip(url: &str) -> Ipv4Addr {
    debug!(target: "cloudflare_dyndns", "Querying current IP: GET {}", url);
    let resp = reqwest::get(url).await.expect("reaching IP server");
    let ip_str = resp.text().await.expect("current IP body");
    debug!(target: "cloudflare_dyndns", "Current IP is {}", ip_str);
    return ip_str.parse().expect("parsing returned IP");
}

fn create_cloudflare_client(auth_token: String) -> Client {
    return Client::new(
        Credentials::UserAuthToken { token: auth_token },
        HttpApiClientConfig::default(),
        Environment::Production,
    )
    .expect("failed to create client");
}

async fn get_recorded_ip(client: &Client, zone_id: &str, domain_name: &str) -> SimpleDnsRecord {
    debug!(target: "cloudflare_dyndns", "Querying DNS record for zone {} and domain {}", zone_id, domain_name);
    let mut list_params = dns::ListDnsRecordsParams::default();
    list_params.name = Some(domain_name.to_string());

    let list = dns::ListDnsRecords {
        zone_identifier: zone_id,
        params: list_params,
    };
    let resp = client.request(&list).await.expect("listing records");

    if resp.result.len() != 1 {
        panic!("Got more than 1 record in response");
    }
    let dns_record = resp.result.into_iter().next().unwrap();

    match dns_record.content {
        dns::DnsContent::A { content } => {
            debug!(target: "cloudflare_dyndns", "IP in DNS record is {} (id: {})", &content, dns_record.id);
            return SimpleDnsRecord {
                name: dns_record.name,
                id: dns_record.id,
                ttl: dns_record.ttl,
                proxied: dns_record.proxied,
                ip: content,
            };
        }
        content => {
            panic!("Expected an A record, received {:?}", content);
        }
    }
}

async fn update_recorded_ip(client: &Client, zone_id: &str, dns_record: SimpleDnsRecord) {
    debug!(target: "cloudflare_dyndns", "Updating DNS record");
    let update = dns::UpdateDnsRecord {
        zone_identifier: zone_id,
        identifier: dns_record.id.as_str(),
        params: dns::UpdateDnsRecordParams {
            name: dns_record.name.as_str(),
            proxied: Some(dns_record.proxied),
            ttl: Some(dns_record.ttl),
            content: dns::DnsContent::A {
                content: dns_record.ip,
            },
        },
    };
    client
        .request(&update)
        .await
        .expect("updating IP on CloudFlare");
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let matches = App::new("cloudflare_dyndns")
        .version("0.1")
        .author("Arjun Guha <arjun@guha-anderson.com>")
        .about("A dynamic DNS client for CloudFlare")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("configuration file")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let conf_filename = matches.value_of("config").unwrap();
    let conf = config::Config::from_file(conf_filename);

    let current_ip = get_current_ip(conf.ip_query_addess.as_str()).await;
    let cloudflare = create_cloudflare_client(conf.cloudflare_auth_token);
    let mut dns_record = get_recorded_ip(
        &cloudflare,
        conf.zone_identifier.as_str(),
        conf.domain_name.as_str(),
    )
    .await;
    if dns_record.ip != current_ip {
        info!(target: "cloudflare_dyndns", "Changing IP address from {} to {}", dns_record.ip, current_ip);
        dns_record.ip = current_ip;
        update_recorded_ip(&cloudflare, conf.zone_identifier.as_str(), dns_record).await;
    }
}
