import { ControllerBaseProxy } from 'bizify';
import { systemService } from '../../services/system.service';
import { ErrorInfo } from '../../typings';
import { SystemInfo } from '../../typings/system';
import { formatKB, formatTimeAgo } from '../../utils';

type Data = {
  loaded: boolean;
  errorInfo: ErrorInfo | null;
  systemInfoRows: string[][];
};

export class SystemCtrl extends ControllerBaseProxy<Data> {
  $data(): Data {
    return {
      loaded: false,
      errorInfo: null,
      systemInfoRows: [],
    };
  }

  services = {
    query: this.$buildService<SystemInfo>(systemService.query.bind(systemService)),
  };

  init = async () => {
    try {
      await this.refresh();
    } catch (err: any) {
      this.data.errorInfo = { message: err.message };
    } finally {
      this.data.loaded = true;
    }
  };

  refresh = async () => {
    const systemInfo = await this.services.query.execute();

    this.data.systemInfoRows = [
      ['Host Name', systemInfo.system_hostname],
      ['System', `${systemInfo.system_name} ${systemInfo.system_os_version}_${systemInfo.system_kernel_version}`],
      ['Uptime', formatTimeAgo(systemInfo.uptime)],
      ['Memory Usage', `${formatKB(systemInfo.free_memory)} / ${formatKB(systemInfo.total_memory)}`],
      ['Load Avg', systemInfo.load_average.map(item => item.toFixed(2)).join(', ')],
      ['Processors', systemInfo.processors_count.toString()],
    ];
  };
}
