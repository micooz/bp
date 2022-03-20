import { ServiceBase } from './base';

export class ConfigService extends ServiceBase {
  constructor() {
    super({
      prefix: '/api/config',
    });
  }

  async query() {
    return this.get('/query');
  }

  async query_acl() {
    return this.get('/query_acl');
  }

  async create() {
    return this.post('/create');
  }

  async create_tls_config(hostname: string) {
    return this.post('/create_tls_config', { hostname });
  }

  async modify(params: { modify_type: 'config' | 'acl', content: string }) {
    return this.post('/modify', { ...params });
  }
}

export const configService = new ConfigService();
