import { ServiceInfo } from '../typings';
import { ServiceBase } from './base';

export class ServiceService extends ServiceBase {
  constructor() {
    super({
      prefix: '/api/service',
    });
  }

  async query(): Promise<ServiceInfo[]> {
    return this.get('/query') as any;
  }

  async start(): Promise<ServiceInfo[]> {
    return this.post('/start') as any;
  }

  async stop() {
    return this.post('/stop');
  }
}

export const serviceService = new ServiceService();
