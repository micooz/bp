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

### Test using Curl

> Both Socks5 and HTTP proxy requests are acceptable by bp client.

```
$ curl -Lx 127.0.0.1:1080 cn.bing.com
$ curl -L --sock5-hostname 127.0.0.1:1080 cn.bing.com
```

## Advanced Usage

### Transparent Proxy

> Transparent Proxy only works on client side

```
$ bp -c --bind 127.0.0.1:1080 --key test
```

### Change Protocol of Transport Layer

Available protocols are:

* `plain`: without encryption.
* `erp`: with AEAD encryption as well as random padding.

```
$ bp -s --bind 127.0.0.1:9000 --key test --protocol plain
```
