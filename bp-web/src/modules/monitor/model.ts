import { ControllerBaseProxy } from 'bizify';
import { monitorService } from '../../services/monitor.service';
import { SystemInfo } from '../../typings/monitor';
import { formatKB, formatTimeAgo } from '../../utils';

type Data = {
  systemInfoRows: string[][];
};

export class MonitorCtrl extends ControllerBaseProxy<Data> {
  $data(): Data {
    return {
      systemInfoRows: [],
    };
  }

  services = {
    querySystemInfo: this.$buildService<SystemInfo>(monitorService.querySystemInfo.bind(monitorService)),
  };

  refresh = async () => {
    const systemInfo = await this.services.querySystemInfo.execute();

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
