export interface ErrorInfo {
  message: string;
  err?: Error;
}

export enum CryptoMethod {
  NONE = '',
  BASE64 = 'base64',
}
