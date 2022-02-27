import { ServiceBase } from './base';

export class ConfigurationService extends ServiceBase {
  constructor() {
    super({
      prefix: '/api/configuration',
    });
  }

  async query() {
    return this.get('/query');
  }

  async create() {
    return this.post('/create');
  }

  async modify(config: any) {
    return this.post('/modify', { ...config });
  }
}

export const configurationService = new ConfigurationService();
