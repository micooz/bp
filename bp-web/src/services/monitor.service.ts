import { ServiceBase } from './base';

export class MonitorService extends ServiceBase {
  constructor() {
    super({
      prefix: '/api/monitor',
    });
  }

  async querySystemInfo() {
    return this.get('/system/info');
  }
}

export const monitorService = new MonitorService();
