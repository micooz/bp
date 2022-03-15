import { ServiceBase } from './base';

export class LogService extends ServiceBase {
  constructor() {
    super({
      prefix: '/api/logging',
    });
  }

  async tail() {
    return this.get('/tail');
  }
}

export const logService = new LogService();
