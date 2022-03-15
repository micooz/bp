export interface SystemInfo {
  system_name: string;
  system_hostname: string;
  system_kernel_version: string;
  system_os_version: string;
  uptime: number;
  free_memory: number;
  total_memory: number;
  processors_count: number;
  load_average: [number, number, number];
}
