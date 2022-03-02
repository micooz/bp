import { useController } from 'bizify';
import { useRef } from 'react';
import { Button, Caption, ErrorBlock } from '../../components';
import { useMount, useUnmount } from '../../hooks';
import { LogCtrl } from './model';
import './index.css';

export const Logs: React.FC<{}> = () => {
  const $dom = useRef<HTMLTextAreaElement>(null);
  const vm = useController<LogCtrl>(LogCtrl);
  const { data: vmData, services } = vm;

  useMount(() => vm.init($dom));

  useUnmount(vm.onDestroy);

  if (!vmData.loaded) {
    return <span className="AnimatedEllipsis" />;
  }

  return (
    <div className="logs">
      <Caption extra={
        <Button
          size="small"
          loading={services.tail.loading}
          onClick={vm.handleRefresh}
        >
          Refresh
        </Button>
      }>
        bp.log
      </Caption>
      <ErrorBlock className="mb-2" errorInfo={vmData.errorInfo} />
      <textarea
        ref={$dom}
        className="form-control logs-textarea"
        style={{ height: '65vh', width: '100%' }}
        value={vmData.log}
        readOnly
      />
    </div>
  );
};
