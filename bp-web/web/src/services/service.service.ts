import { ServiceBase } from './base';

export class ServiceService extends ServiceBase {
  constructor() {
    super({
      prefix: '/api/service',
    });
  }

  async query() {
    return this.get('/query');
  }

  async start() {
    return this.post('/start');
  }

  async stop() {
    return this.post('/stop');
  }
}

export const serviceService = new ServiceService();
