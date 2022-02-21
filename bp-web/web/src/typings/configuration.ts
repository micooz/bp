export interface Configuration {
  bind: string;
  with_basic_auth: string;
  server_bind: string;
  pac_bind: string;
  key: string;
  encryption: string;
  acl: string;
  pin_dest_addr: string;
  udp_over_tcp: boolean;
  dns_server: string;
  tls: boolean;
  quic: boolean;
  quic_max_concurrency: number;
  tls_cert: string;
  monitor: string;
}
