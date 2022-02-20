import { ServiceBase } from './base';
import { Configuration } from '../typings';

export class ConfigurationService extends ServiceBase {
  constructor() {
    super({
      prefix: '/api/configuration',
    });
  }

  async query() {
    return this.get<Configuration>('/query');
  }

  async create() {
    return this.post<Configuration>('/create');
  }

  async modify() {
    return this.post<Configuration>('/modify');
  }
}
