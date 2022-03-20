import { useController } from 'bizify';
import React, { useRef } from 'react';
import { Button, Caption, Checkbox, ErrorBlock, TextArea } from '../../components';
import { useMount, useUnmount } from '../../hooks';
import { LogCtrl } from './model';
import './index.css';

export const Logs: React.FC<{}> = () => {
  const $dom = useRef<HTMLTextAreaElement>(null);
  const vm = useController<LogCtrl>(LogCtrl);
  const { data: vmData, services } = vm;

  useMount(() => vm.init($dom));

  useUnmount(vm.onDestroy);

  return (
    <div className="logs">
      <ErrorBlock className="mb-3">{services.tail.error?.message}</ErrorBlock>
      <Caption extra={
        <div className="d-flex">
          <Button
            size="small"
            loading={services.tail.loading}
            onClick={vm.handleRefresh}
          >
            Refresh
          </Button>
          <Checkbox
            name=""
            className="ml-2"
            checked={vmData.autoRefresh}
            onChange={vm.handleAutoRefreshClick}
          >
            <h6>Auto Refresh</h6>
          </Checkbox>
        </div>
      }>
        bp.log
      </Caption>
      <TextArea
        ref={$dom}
        className="logs-textarea"
        style={{ height: '65vh' }}
        value={vmData.log}
        readonly
      />
    </div>
  );
};
