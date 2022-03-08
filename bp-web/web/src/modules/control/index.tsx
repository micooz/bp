import { useController } from 'bizify';
import { Box, Button, DotStatus, ErrorBlock } from '../../components';
import { useMount } from '../../hooks';
import { ControlCtrl } from './model';

export const Control: React.FC<{}> = () => {
  const vm = useController<ControlCtrl>(ControlCtrl);
  const { data: vmData, services } = vm;

  useMount(vm.init);

  return (
    <div className="control">
      <Box
        title="Process"
        className="mb-3"
        extra={
          <Button
            type="primary"
            size="small"
            disabled={!!services.query.error || services.query.loading}
            loading={services.start.loading || services.stop.loading}
            onClick={vm.toggleService}
          >
            {vmData.online ? 'Stop' : 'Start'}
          </Button>
        }
        condensed
      >
        <DotStatus status={vmData.online ? 'on' : 'off'}>
          {vmData.online ? `running at ${vmData.serviceInfo?.bind_host}:${vmData.serviceInfo?.bind_port}` : 'not running'}
        </DotStatus>
      </Box>

      <ErrorBlock className="ml-3 mr-3">
        {!vmData.online && (
          services.query.error?.message ||
          services.start.error?.message ||
          services.query.error?.message
        )}
      </ErrorBlock>
    </div>
  );
};
