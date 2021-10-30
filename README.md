[![Build & Release](https://github.com/micooz/bp/actions/workflows/build-release.yml/badge.svg)](https://github.com/micooz/bp/actions/workflows/build-release.yml)
[![Build & Test](https://github.com/micooz/bp/actions/workflows/build-test.yml/badge.svg)](https://github.com/micooz/bp/actions/workflows/build-test.yml)

# bp

Lightweight and efficient proxy written in pure Rust.

## Features

* Running on all platform and all CPU architecture.
* Socks5/HTTP/HTTPS/DNS proxy all in one port.
* Support UDP over TCP.
* Work with Linux Firewall(by iptables).
* White list control.

## Usage

Please check -h/--help for more information, or check out [USAGE](USAGE.txt).

```
$ bp -h
```

## Examples

### Run as client

```
$ bp -c --key key --server-bind <host:port>
```

### Run as server

```
$ bp -s --bind 127.0.0.1:9000 --key key
```

### Test with Curl

> Both Socks5 and HTTP Proxy requests are acceptable by bp client.

```
$ curl -Lx 127.0.0.1:1080 cn.bing.com
$ curl -L --sock5-hostname 127.0.0.1:1080 cn.bing.com
```

## Advanced Usage

### Relay Directly

If not set `--server-bind`, bp will relay directly.

```
$ bp -c
```

### UDP over TCP

```
$ bp -c --key key --udp-over-tcp --server-bind <host:port>
```

### Pin Destination Address

> NOTE: this is usually for testing via **iperf**

```
$ bp -c --force-dest-addr <host:port>
```

### Change Transport Protocol

The transport protocol can be switched between bp client and bp server, available protocols are:

* `plain`: without encryption.
* `erp`: with AEAD encryption as well as random padding. (default)

```
$ bp -s --bind 127.0.0.1:9000 --key test --protocol plain
```


### Proxy White List

```
$ bp -c --proxy-white-list /path/to/list.txt
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

## Monitor (Experimental)

bp executable compiled with `--features="monitor"` expose a TCP control port which can be used for remote monitoring.

### use telnet

Use `telnet` connect to control port then follow the prompt:

```
$ telnet <bp_host> <bp_monitor_port>
```
