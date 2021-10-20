# bp

Lightweight and efficient proxy written in pure Rust.

## Features

* Running on all platform and all CPU architecture.
* Socks5/HTTP/HTTPS/DNS proxy all in one port.
* Support UDP over TCP.
* Work with Linux Firewall(by iptables).
* White list control.

## Usage

Please check -h/--help first.

```
$ bp -h
bp 1.0.0-alpha.0

Lightweight and efficient proxy written in pure Rust

USAGE:
    bp [OPTIONS]

OPTIONS:
    -b, --bind <BIND>
            local service bind address [default: 127.0.0.1:1080]

    -c, --client
            run as client

        --dns-server <DNS_SERVER>
            DNS server address [default: 8.8.8.8:53]

        --force-dest-addr <FORCE_DEST_ADDR>
            force all incoming data relay to this destination, usually for testing [default: false]

    -h, --help
            Print help information

    -k, --key <KEY>
            symmetric encryption key

    -p, --protocol <PROTOCOL>
            protocol used by transport layer between client and server, "plain" or "erp" are
            supported [default: erp]

        --proxy-white-list <PROXY_WHITE_LIST>
            check white list before proxy, pass a file path

    -s, --server
            run as server

        --server-bind <SERVER_BIND>
            bp server bind address, client only. If not set, bp will relay directly

        --udp-over-tcp
            proxy UDP via TCP, client only. Requires --server-bind to be set if true [default:
            false]

    -V, --version
            Print version information
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
