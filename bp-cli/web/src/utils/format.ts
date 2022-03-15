import filesize from 'filesize';
import { format as timeago } from 'timeago.js';

export function formatKB(size: number | string) {
  return filesize(+size * 1e3);
}

export function formatTimeAgo(seconds: number) {
  return timeago(Date.now() - seconds * 1e3);
}
