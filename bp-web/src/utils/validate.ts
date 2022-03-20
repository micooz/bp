export function isValidJson(value: any) {
  try {
    JSON.parse(value);
    return true;
  } catch (err) {
    return false;
  }
}
