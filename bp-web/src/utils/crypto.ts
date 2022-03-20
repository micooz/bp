import { Base64 } from 'js-base64';

export interface ICrypto<T = any> {
  encrypt(plaintext: T): T;
  decrypt(ciphertext: T): T;
}

export class Base64Crypto implements ICrypto<string> {
  encrypt(plaintext: string) {
    return Base64.encode(plaintext);
  }
  decrypt(ciphertext: string) {
    return Base64.decode(ciphertext);
  }
}
