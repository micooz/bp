export interface RequestOptions {
  method?: 'GET' | 'POST',
  url: string;
  data?: Record<string, any>;
}

export async function httpRequest<T>(opts: RequestOptions): Promise<T> {
  let url = opts.url;
  let args: RequestInit = {};

  if (opts.method === 'GET') {
    const search = new URLSearchParams(opts.data).toString();
    if (search) {
      url += `?${search}`;
    }
  } else if (opts.method === 'POST') {
    args = {
      body: opts.data ? JSON.stringify(opts.data) : undefined,
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

  return promise
    .then<any>(res => {
      if (res.status !== 200) {
        // eslint-disable-next-line no-throw-literal
        throw { status: res.status, data: null };
      }
      return res.json();
    });
}
