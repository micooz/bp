import { CryptoMethod } from '../typings';

declare global {
  interface Window {
    __VERSION__: string;
    __RUST_VERSION__: string;
    __RUN_TYPE__: string;
    __CRYPTO_METHOD__: CryptoMethod;
  }
}

export const VERSION = window.__VERSION__;
export const RUST_VERSION = window.__RUST_VERSION__;
export const RUN_TYPE = window.__RUN_TYPE__;
export const RUN_TYPE_CLIENT = window.__RUN_TYPE__ === 'CLIENT';
export const RUN_TYPE_SERVER = window.__RUN_TYPE__ === 'SERVER';
export const CRYPTO_METHOD = window.__CRYPTO_METHOD__;
