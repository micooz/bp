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
* Support custom proxy whitelist and PAC service.
* Work with Linux Firewall(via iptables).

## 2.0 Roadmap

- [x] Refine CLI to multiple subcommands
- [x] TLS transport layer
- [x] PAC Service
- [ ] Improve performance of I/O reader
- [ ] HTTP Client Proxy Authorization
- [ ] HTTPS Client Proxy with Authorization
- [ ] Tracer & Monitor Service
- [ ] Web GUI
- [ ] iOS GUI
- [ ] Proxy Rule List to PAC

## Basic Usages

Please check -h/--help first, or see [USAGE](usage).

```
$ bp -h
```

### Run as Client

```
$ bp client --key key --server-bind <host:port>
```

### Run as Server

```
$ bp server --bind 127.0.0.1:9000 --key key
```

### Curl Test

> Both Socks5 and HTTP Proxy requests are acceptable by bp client on the same port.

Assume bp client is running at `127.0.0.1:1080`:

```
$ curl --sock5-hostname 127.0.0.1:1080 cn.bing.com
$ curl -x 127.0.0.1:1080 cn.bing.com
```

## Advanced Usages

### Relay Directly

If not set `--server-bind`, bp will relay directly.

```
$ bp client
```

### UDP over TCP

```
$ bp client --key key --udp-over-tcp --server-bind <host:port>
```

### Enable TLS

First, generate self-signed certificates:

```
$ bp generate --certificate --hostname localhost
```

Then, provide bp server with Certificate and Private Key:

```
$ bp server --tls --tls-cert <cert_path> --tls-key <key_path> <other_options>
```

Finally, provide bp client with Certificate only:

```
$ bp client --tls --tls-cert <cert_path> <other_options>
```

### Enable QUIC

[QUIC](https://quicwg.github.io/) is a transport protocol based on UDP and TLS, it force use TLS, so we should first generate TLS Certificate and Private Key. The steps are almost the same as **Enable TLS**, just need replace `--tls` to `--quic`.

### Pin Destination Address

> NOTE: this is usually for testing via **iperf**

```
$ bp client --force-dest-addr <host:port>
```

### Change Protocol

The protocol can be switched between bp client and bp server, available protocols are:

* `plain`: without encryption.
* `erp`: with AEAD encryption as well as random padding. (default)

```
$ bp client --bind 127.0.0.1:9000 --key test --protocol plain
```

### Proxy White List

```
$ bp client --proxy-white-list /path/to/list.txt
```

Assume that the white list file contains the following rules:

```
example.com
~example.com
!example.com
#example.com
```

The prefixes means:

* `<no prefix>`: exactly match, matched domain name will be proxy
* `~`: fuzzy match, matched domain name will be proxy
* `!`: not match, matched domain name will NOT be proxy
* `#`: comment string, will skip matching

> Higher priority for later rules

### PAC Service

By adding `--pac-bind`, you can start a PAC service at specified address while bp client started. The content of `proxy.pac` is based on your `--proxy-white-list`, you must prepare this file first.

```
$ bp client --proxy-white-list /path/to/list.txt --pac-bind <host:port>
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
