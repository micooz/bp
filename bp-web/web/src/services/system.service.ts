import { ServiceBase } from './base';

export class SystemService extends ServiceBase {
  constructor() {
    super({
      prefix: '/api/system',
    });
  }

  async query() {
    return this.get('/info');
  }
}

export const systemService = new SystemService();
