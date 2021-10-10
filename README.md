# bp

Lightweight and efficient proxy written in pure Rust.

## Usage

Please check -h/--help first.

```
$ bp -h
bp 0.1.0

Lightweight and efficient proxy written in pure Rust

USAGE:
    bp [FLAGS] [OPTIONS]

FLAGS:
    -c, --client        run as client
        --enable-udp    enable udp relay
    -h, --help          Print help information
    -s, --server        run as server
    -V, --version       Print version information

OPTIONS:
    -b, --bind <BIND>
            local service bind address [default: 127.0.0.1:1080]

        --force-dest-addr <FORCE_DEST_ADDR>
            force all incoming data relay to this destination, usually for testing

    -k, --key <KEY>
            symmetric encryption key

        --protocol <PROTOCOL>
            protocol used by transport layer between client and server, "plain" or "erp" are
            supported [default: erp]

        --proxy-list-path <PROXY_LIST_PATH>
            check white list before proxy

        --server-bind <SERVER_BIND>
            bp server bind address, client only. if not set, bp will relay directly
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

### Enable UDP Relay

```
$ bp -c --enable-udp
```

### Pin Destination Address

> NOTE: this is usually for testing via **iperf**

```
$ bp -c --force-dest-addr <host>:<port>
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
$ bp -c --proxy-list-path /path/to/list.txt
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
