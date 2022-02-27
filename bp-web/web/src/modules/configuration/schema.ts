import { RUN_TYPE_CLIENT, RUN_TYPE_SERVER } from '../../common';
import { FormItem } from '../../components';

interface FormSchema {
  basic: FormItem[];
  advanced: FormItem[];
}

const ClientSchema: FormSchema = {
  basic: [
    {
      name: 'bind',
      key: 'bind',
      type: 'text',
      required: true,
      placeholder: 'host:port',
      description: 'Local service bind address [default: 127.0.0.1:1080]',
    },
    {
      name: 'with_basic_auth',
      key: 'with_basic_auth',
      type: 'text',
      placeholder: 'user:pass',
      description: 'Basic authorization required for HTTP Proxy, e,g. "user:pass" [default: <empty>]',
    },
    {
      name: 'server_bind',
      key: 'server_bind',
      type: 'text',
      placeholder: 'host:port',
      description: 'Server bind address. If not set, bp will relay directly [default: <empty>]',
    },
    {
      name: 'key',
      key: 'key',
      type: 'text',
      required_if: 'server_bind',
      description: 'Symmetric encryption key, required if --server-bind is set [default: <empty>]',
    },
    {
      name: 'encryption',
      key: 'encryption',
      type: 'select',
      required_if: 'server_bind',
      description: 'Data encryption method, e.g, "plain" or "erp"',
      options: [
        {
          label: 'plain',
          value: 'plain',
        },
        {
          label: 'erp',
          value: 'erp',
        },
      ],
    },
    {
      name: 'tls',
      key: 'tls',
      type: 'boolean',
      description: 'Enable TLS for Transport Layer [default: false]',
    },
    {
      name: 'quic',
      key: 'quic',
      type: 'boolean',
      description: 'Enable QUIC for Transport Layer [default: false]',
    },
    {
      name: 'quic_max_concurrency',
      key: 'quic_max_concurrency',
      type: 'number',
      min: 1,
      max: 65535,
      description: 'The max number of QUIC connections [default: Infinite]',
    },
    {
      name: 'tls_cert',
      key: 'tls_cert',
      type: 'text',
      required_if: ['tls', 'quic'],
      description: 'Certificate for QUIC or TLS [default: <empty>]',
    },
  ],
  advanced: [
    {
      name: 'dns_server',
      key: 'dns_server',
      type: 'text',
      placeholder: 'host:port',
      description: 'DNS server address',
    },
    {
      name: 'acl',
      key: 'acl',
      type: 'text',
      required_if: 'pac_bind',
      description: 'Check ACL before proxy, pass a file path [default: <empty>]',
    },
    {
      name: 'pac_bind',
      key: 'pac_bind',
      type: 'text',
      placeholder: 'host:port',
      description: 'Start a PAC server at the same time, requires --acl [default: <empty>]',
    },
  ],
};

const ServerSchema: FormSchema = {
  basic: [
    {
      name: 'bind',
      key: 'bind',
      type: 'text',
      required: true,
      placeholder: 'host:port',
      description: 'Local service bind address [default: 127.0.0.1:1080]',
    },
    {
      name: 'key',
      key: 'key',
      type: 'text',
      required_if: 'server_bind',
      description: 'Symmetric encryption key, required if --server-bind is set [default: <empty>]',
    },
    {
      name: 'encryption',
      key: 'encryption',
      type: 'select',
      required_if: 'server_bind',
      description: 'Data encryption method, e.g, "plain" or "erp"',
      options: [
        {
          label: 'plain',
          value: 'plain',
        },
        {
          label: 'erp',
          value: 'erp',
        },
      ],
    },
    {
      name: 'tls',
      key: 'tls',
      type: 'boolean',
      description: 'Enable TLS for Transport Layer [default: false]',
    },
    {
      name: 'quic',
      key: 'quic',
      type: 'boolean',
      description: 'Enable QUIC for Transport Layer [default: false]',
    },
    {
      name: 'tls_cert',
      key: 'tls_cert',
      type: 'text',
      required_if: ['tls', 'quic'],
      description: 'Certificate for QUIC or TLS [default: <empty>]',
    },
    {
      name: 'tls_key',
      key: 'tls_key',
      type: 'text',
      required_if: ['tls', 'quic'],
      description: 'Private key file for QUIC or TLS [default: <empty>]',
    },
  ],
  advanced: [
    {
      name: 'dns_server',
      key: 'dns_server',
      type: 'text',
      placeholder: 'host:port',
      description: 'DNS server address',
    },
    {
      name: 'acl',
      key: 'acl',
      type: 'text',
      description: 'Check ACL before proxy, pass a file path [default: <empty>]',
    },
  ],
};

let formSchema: FormSchema;

if (RUN_TYPE_CLIENT) {
  formSchema = ClientSchema;
}

if (RUN_TYPE_SERVER) {
  formSchema = ServerSchema;
}

export { formSchema };
