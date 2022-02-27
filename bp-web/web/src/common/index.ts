// @ts-ignore
export const VERSION = window.__VERSION__ as string;

// @ts-ignore
export const RUN_TYPE_CLIENT = window.__RUN_TYPE__ === 'CLIENT';
// @ts-ignore
export const RUN_TYPE_SERVER = window.__RUN_TYPE__ === 'SERVER';

let RUN_TYPE: string;

if (RUN_TYPE_CLIENT) RUN_TYPE = 'CLIENT';
if (RUN_TYPE_SERVER) RUN_TYPE = 'SERVER';

export { RUN_TYPE };
