import { Base64Crypto, httpRequest } from '../utils';
import { CRYPTO_METHOD } from '../common';
import { CryptoMethod } from '../typings';

interface ServiceBaseOptions {
  prefix: string;
}

interface Response<T> {
  success: boolean;
  errorMessage: string;
  data: T;
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

  private async http<T>(method: 'GET' | 'POST', path: string, data: any): Promise<string | T> {
    const res: string | Response<T> = await httpRequest({
      method,
      url: `${this.prefix}${path}`.replace(/\/\//g, '/'),
      data,
      crypto: this.getCrypto(),
    });

    if (typeof res === 'string') {
      return res;
    }

    if (res.success === false) {
      throw res;
    }

    return res.data;
  }

  private getCrypto() {
    switch (CRYPTO_METHOD) {
      case CryptoMethod.BASE64:
        return new Base64Crypto();
      case CryptoMethod.NONE:
      default:
        break;
    }
  }
}
