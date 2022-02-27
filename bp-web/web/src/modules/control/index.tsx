import { useController } from 'bizify';
import { Button, DotStatus, ErrorBlock } from '../../components';
import { useMount } from '../../hooks';
import { ControlCtrl } from './model';

export const Control: React.FC<{}> = () => {
  const vm = useController<ControlCtrl>(ControlCtrl);
  const { data: vmData, services } = vm;

  useMount(vm.init);

  return (
    <div className="control">
      <div className="m-3 d-flex flex-justify-between flex-items-center">
        <DotStatus status={vmData.online ? 'on' : 'off'}>
          {vmData.online ? `running at ${vmData.serviceInfo?.bind_host}:${vmData.serviceInfo?.bind_port}` : 'not running'}
        </DotStatus>
        <Button
          type="primary"
          size="small"
          disabled={services.query.loading}
          loading={services.start.loading || services.stop.loading}
          onClick={vm.toggleService}
        >
          {vmData.online ? 'Stop' : 'Start'}
        </Button>
      </div>
      <ErrorBlock className="ml-3 mr-3" errorInfo={vmData.errorInfo} />
    </div>
  );
};
