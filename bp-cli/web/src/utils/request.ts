export interface RequestOptions {
  method?: 'GET' | 'POST',
  url: string;
  data?: Record<string, any>;
}

export interface Response<T> {
  success: boolean;
  message?: string;
  data?: T;
}

export async function httpRequest<T>(opts: RequestOptions): Promise<Response<T>> {
  let url = opts.url;
  let args: RequestInit = {};

  if (opts.method === 'GET') {
    const search = new URLSearchParams(opts.data).toString();
    if (search) {
      url += `?${search}`;
    }
  } else if (opts.method === 'POST') {
    args = {
      body: opts.data ? JSON.stringify(opts.data, null, 2) : undefined,
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

  if (!res.ok) {
    const message = (await res.text()) || res.statusText;
    // eslint-disable-next-line no-throw-literal
    throw { success: false, message };
  }

  let data = null;

  const type = res.headers.get('content-type');

  if (type?.startsWith('application/json')) {
    data = await res.json();
  } else {
    data = await res.text();
  }

  return { success: true, data };
}
