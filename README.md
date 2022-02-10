[![Build & Release](https://github.com/micooz/bp/actions/workflows/build-release.yml/badge.svg)](https://github.com/micooz/bp/actions/workflows/build-release.yml)
[![Build & Test](https://github.com/micooz/bp/actions/workflows/build-test.yml/badge.svg)](https://github.com/micooz/bp/actions/workflows/build-test.yml)
[![codecov](https://codecov.io/gh/micooz/bp/branch/main/graph/badge.svg?token=7FCI8FK1UL)](https://codecov.io/gh/micooz/bp)

# bp

bp is a set of advanced and efficient proxy tools written in pure Rust.

## Features

* Cross-platform, of course. Linux/Windows/macOS and others.
* Support Socks5/HTTP/HTTPS Proxy Protocols.
* Support proxy non-proxy protocols, for example: HTTP/HTTPS/DNS.
* Support multiple transport protocols, for example: TLS/QUIC.
* Support ACL(Access Control List) and PAC(Proxy Auto Config) service.
* Work with Linux Firewall(via iptables).

## 2.0 Roadmap

- [x] Refine CLI to multiple subcommands
- [x] TLS transport layer
- [x] Configuration generators
- [x] HTTP Proxy Basic Authorization
- [x] PAC Service based on access control list
- [x] Enhance acl, support for bp server
- [ ] Improve performance of I/O reader
- [ ] Tracer & Monitor Service
- [ ] Web GUI

## Planned Features

- [ ] HTTPS Client Proxy with Authorization
- [ ] Deploy to iOS/Android

## Basic Usages

Please check -h/--help first, or see [USAGE](usages).

```
$ bp -h
```

### Generate Configuration

> It's easier for newcomers to use configuration file.

Run the following commands to create bp configurations automatically, you can generate **YAML** or **JSON** file by changing the file extension.

```
$ bp generate --config client.json --config-type client
$ bp generate --config server.json --config-type server
```

Modify configuration items as needed.

### Run as Client

```
$ bp client --config client.json
```

### Run as Server

```
$ bp server --config server.json
```

### Test with bp-test

You can use bp-test to check if your configuration is correct.

```
$ bp test --config client.json --http www.google.com:80
```

### Test with Curl

> Both Socks5 and HTTP Proxy requests are acceptable by bp client on the same port.

Assume bp client is running at `127.0.0.1:1080`:

```
$ curl --sock5-hostname 127.0.0.1:1080 cn.bing.com
$ curl -x 127.0.0.1:1080 cn.bing.com
```

## Advanced Usages

> The following guides use CLI options instead of configuration file.

### No Proxy

If not set `--server-bind`, bp will relay directly.

```
$ bp client
```

### Encryption Method

```
$ bp client --bind 127.0.0.1:9000 --key test --encryption <method>
```

`<method>` can be:

* `plain`: without encryption.
* `erp`: AEAD encryption with random padding. (default)

### Access Control

Access Control works for both client and server side.

```
$ bp client --acl /path/to/acl.txt
$ bp server --acl /path/to/acl.txt
```

**White List Example**

```
example.com
example1.com
```

Or add `[Deny]` and `[Allow]` pair:

```
[Deny]
*:*

[Allow]
example.com
```

**Black List Example**

```
[Allow]
*:*

[Deny]
example.com
```

**Mixed Example**

You can mix use `[Allow]` and `[Deny]`.

```
[Allow]
example.com

[Deny]
example1.com

[Allow]
example1.com # example1.com is allowed again
```

The format of each rule is `[<hostname>]:[<port>]`, for example:

```
*:*
example.com
example.com:*
example.com:80
```

Each rule can add a prefix to change match behavior:

* `~`: fuzzy match, e,g. `~example.com:443` will match `*example.com*:443`
* `#`: comment string, skip matching, e,g. `#example.com`

> Note that:
> 1. The default strategy is **Black List**
> 2. Higher priority for later rules

### PAC Service

By adding `--pac-bind`, you can start a PAC service at specified address while bp client started. The content of `proxy.pac` is generate from your `--acl`, you must prepare this file first.

```
$ bp client --acl /path/to/acl.txt --pac-bind <host:port>
```

### Enable TLS

First, generate self-signed certificates:

```
$ bp generate --certificate --hostname localhost
```

Then, provide bp server with Certificate and Private Key:

```
$ bp server --tls --tls-cert <cert_path> --tls-key <key_path>
```

Finally, provide bp client with Certificate only:

```
$ bp client --tls --tls-cert <cert_path>
```

### Enable QUIC

[QUIC](https://quicwg.github.io/) is a transport protocol based on UDP and TLS, it force use TLS, so we should first generate TLS Certificate and Private Key. The steps are almost the same as **Enable TLS**, just need replace `--tls` to `--quic`.


### UDP over TCP

```
$ bp client --key key --udp-over-tcp --server-bind <host:port>
```

### Pin Destination Address

> NOTE: this is usually for testing via **iperf**

```
$ bp client --pin-dest-addr <host:port>
```

### Linux Router

In order to proxy the traffic of all devices access to a router, you can add iptables rules on router to redirect all http/https traffic to bp, bp will identify the destination address in the traffic and then proxy it.

Add the following rules:

```
iptables -t nat -N BP
iptables -t nat -A BP -d 192.168.0.0/16 -j RETURN
iptables -t nat -A BP -p tcp -j RETURN -m mark --mark 0xff
iptables -t nat -A BP -p tcp -m multiport --dports 80,443 -j REDIRECT --to-ports 1080
iptables -t nat -A PREROUTING -p tcp -j BP
iptables -t nat -A OUTPUT -p tcp -j BP
```
