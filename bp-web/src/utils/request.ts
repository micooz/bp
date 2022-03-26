import type { ICrypto } from './crypto';

export interface RequestOptions {
  method?: 'GET' | 'POST',
  url: string;
  data?: Record<string, any>;
  crypto?: ICrypto;
}

export async function httpRequest(opts: RequestOptions): Promise<any> {
  let url = opts.url;
  let args: RequestInit = {};

  if (opts.method === 'GET') {
    const search = new URLSearchParams(opts.data).toString();

    if (search) {
      const finalSearch = opts.crypto ? opts.crypto.encrypt(search) : search;
      url += `?${finalSearch}`;
    }
  }

  if (opts.method === 'POST') {
    let finalBody = opts.data ? JSON.stringify(opts.data, null, 2) : undefined;

    if (finalBody && opts.crypto) {
      finalBody = opts.crypto.encrypt(finalBody);
    }

    args = {
      body: finalBody,
      headers: {
        'content-type': 'application/json',
      },
      credentials: 'include',
    };
  }

  const promise = fetch(url, {
    method: opts.method || 'GET',
    ...args,
  });

  const res = await promise;

  let type = res.headers.get('content-type');
  let data = await res.text();

  if (opts.crypto && type?.includes('octet-stream')) {
    data = opts.crypto.decrypt(data);
    type = 'application/json';
  }

  if (!res.ok) {
    const message = data || res.statusText;
    throw Error(message);
  }

  if (type?.includes('json')) {
    data = JSON.parse(data);
  }

  return data;
}
