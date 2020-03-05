# DynamicDNS for CloudFlare

A simple program that updates a DNS record in CloudFlare.

# Prerequisites

We need a web service that returns your IP address in the body of a GET request.
The following code is a CloudFlare Worker that does so:

```
addEventListener('fetch', event => {
  event.respondWith(handleRequest(event.request))
})

async function handleRequest(request) {
  let ip = String(request.headers.get('CF-Connecting-IP'));
  return new Response(ip, {status: 200})
}
```

# Configuration

The program expects a JSON configuration file with the following keys:

```
{
    "cloudflare_auth_token": "<api-token>",
    "zone_identifier": "<zone-identifier>",
    "domain_name": "<domain-name>",
    "ip_query_addess": "<address-of-ip-service>"
}
```

- We can generate the `<api-token>` from the CloudFlare web site. Click
  "My Profile" on the top-right corner, go to the "API Tokens" tab, and click
  "Create Token". The new token must grant permission to edit the DNS Zone.

- The `<zone-identifier>` appears on the right-hand column of a website on
  the "Overview" page.

- The `<domain-name>` is the fully qualified domain name that we want to
  update.

- The `<ip_query_address>` is the address of the website that reports your
  IP address.