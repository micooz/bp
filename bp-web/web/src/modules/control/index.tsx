import { useController } from 'bizify';
import { Box, Button, DotStatus, ErrorBlock } from '../../components';
import { useMount } from '../../hooks';
import { ServiceInfo } from '../../typings';
import { ControlCtrl } from './model';
import './index.css';

export const Control: React.FC<{}> = () => {
  const vm = useController<ControlCtrl>(ControlCtrl);
  const { data: vmData, services } = vm;

  useMount(vm.init);

  return (
    <div className="control">
      <ErrorBlock className="mb-3">
        {!vmData.online && (
          services.query.error?.message ||
          services.start.error?.message ||
          services.query.error?.message
        )}
      </ErrorBlock>
      <Box
        title={`Service (${vmData.services.length})`}
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
        <div className="control-body">
          {vmData.services.length === 0 && 'no service running'}
          {vmData.services.map((item, index) => <ServiceItem key={index} service={item} />)}
        </div>
      </Box>
    </div>
  );
};

interface ServiceItemProps {
  service: ServiceInfo;
}

function ServiceItem(props: ServiceItemProps) {
  const { service } = props;

  let str = '';

  if (service.protocol === 'pac') {
    str = `http://${service.bind_host}:${service.bind_port}/proxy.pac`;
  } else {
    str = `${service.protocol}://${service.bind_host}:${service.bind_port}`;
  }

  return (
    <DotStatus className="mb-1" status="on">
      running at {str}
    </DotStatus>
  );
}
