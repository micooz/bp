import { useController } from 'bizify';
import { SystemCtrl } from './model';
import { Button, Caption, ErrorBlock, Table } from '../../components';
import { useMount } from '../../hooks';

export const System: React.FC<{}> = () => {
  const vm = useController<SystemCtrl>(SystemCtrl);
  const { data: vmData, services } = vm;

  useMount(vm.init);

  if (!vmData.loaded) {
    return null;
  }

  if (vmData.errorInfo) {
    return <ErrorBlock errorInfo={vmData.errorInfo} />;
  }

  return (
    <div className="system">
      <Caption extra={
        <Button loading={services.query.loading} size="small" onClick={vm.refresh}>
          Refresh
        </Button>
      }>
        System
      </Caption>
      <Table rows={vmData.systemInfoRows} />
    </div>
  );
};
