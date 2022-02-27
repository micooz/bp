import { ServiceBase } from './base';

export class SecurityService extends ServiceBase {
  constructor() {
    super({
      prefix: '/api/security',
    });
  }

  async query() {
    return this.get('/query');
  }

  async create(hostname: string) {
    return this.post('/create', { hostname });
  }
}

export const securityService = new SecurityService();
