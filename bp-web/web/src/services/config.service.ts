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

  async create() {
    return this.post('/create');
  }

  async create_tls_config() {
    return this.post('/create_tls_config');
  }

  async modify(config: any) {
    return this.post('/modify', { ...config });
  }
}

export const configService = new ConfigService();
