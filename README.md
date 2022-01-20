[![Build & Release](https://github.com/micooz/bp/actions/workflows/build-release.yml/badge.svg)](https://github.com/micooz/bp/actions/workflows/build-release.yml)
[![Build & Test (Self Hosted)](https://github.com/micooz/bp/actions/workflows/build-test-self-hosted.yml/badge.svg)](https://github.com/micooz/bp/actions/workflows/build-test-self-hosted.yml)

# bp

Lightweight and efficient proxy written in pure Rust.

## Features

* Running on all platform and all CPU architecture.
* Handle Socks5/HTTP/HTTPS/DNS requests at once in a single port.
* Support UDP over TCP.
* Support QUIC.
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

## Monitor (Experimental)

bp executable compiled with `--features="monitor"` expose a TCP control port which can be used for remote monitoring.

### use telnet

Use `telnet` connect to control port then follow the prompt:

```
$ telnet <bp_host> <bp_monitor_port>
```
