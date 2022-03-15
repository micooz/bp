import { useController } from 'bizify';
import { MonitorCtrl } from './model';
import { Button, Caption, ErrorBlock, Table } from '../../components';
import { useMount } from '../../hooks';

export const Monitor: React.FC<{}> = () => {
  const vm = useController<MonitorCtrl>(MonitorCtrl);
  const { data: vmData, services } = vm;

  useMount(vm.refresh);

  return (
    <div className="monitor">
      <ErrorBlock className="mb-3">{services.querySystemInfo.error?.message}</ErrorBlock>

      <div className="monitor-system mb-3">
        <Caption
          extra={
            <Button loading={services.querySystemInfo.loading} size="small" onClick={vm.refresh}>
              Refresh
            </Button>
          }
        >
          System Info
        </Caption>
        <Table rows={vmData.systemInfoRows} />
      </div>

      <div className="monitor-metrics mb-2">
        <Caption>
          Metrics
        </Caption>
      </div>
    </div>
  );
};
