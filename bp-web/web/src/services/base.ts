import { httpRequest } from '../utils';

interface ServiceBaseOptions {
  prefix: string;
}

export class ServiceBase {
  private prefix: string;

  constructor(opts: ServiceBaseOptions) {
    this.prefix = opts.prefix;
  }

  protected async get<T>(path: string, data?: Record<string, any>) {
    return this.http<T>('GET', path, data);
  }

  protected async post<T>(path: string, data?: Record<string, any>) {
    return this.http<T>('POST', path, data);
  }

  private async http<T>(method: 'GET' | 'POST', path: string, data: any) {
    const res = await httpRequest<T>({
      method,
      url: `${this.prefix}${path}`.replace(/\/\//g, '/'),
      data,
    });
    return res.data;
  }
}