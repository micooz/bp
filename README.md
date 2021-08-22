# bp

Lightweight and efficient proxy written in pure Rust.

## Basic Usage

### Run as client

```
$ bp -c --bind 127.0.0.1:1080 --key test --server-host somewhere --server-port 9090
```

### Run as server

```
$ bp -s --bind 127.0.0.1:9000 --key test
```

### Test with Curl

> Both Socks5 and HTTP proxy requests are acceptable by bp client.

```
$ curl -Lx 127.0.0.1:1080 cn.bing.com
$ curl -L --sock5-hostname 127.0.0.1:1080 cn.bing.com
```

## Advanced Usage

### Transparent Proxy

You can turn bp `client` into transparent proxy by omitting `--server-host` and `--server-port` options.

> Transparent Proxy only works on client side

```
$ bp -c --bind 127.0.0.1:1080 --key test
```

### Change Transport Protocol

The transport protocol can be switched between bp client and bp server, available protocols are:

* `plain`: without encryption.
* `erp`: with AEAD encryption as well as random padding. (default)

```
$ bp -s --bind 127.0.0.1:9000 --key test --protocol plain
```

## Monitor

bp executable compiled with `--features="monitor"` expose a TCP control port which can be used for remote monitoring.

### use telnet

Use `telnet` connect to control port then follow the prompt:

```
$ telnet <bp_host> <bp_monitor_port>
```
