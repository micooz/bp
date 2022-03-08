import React from 'react';
import { useController } from 'bizify';
import { Button, Caption, ErrorBlock, FormBuilder, TextArea } from '../../components';
import { useMount } from '../../hooks';
import { RUN_TYPE_SERVER } from '../../common';
import { CodeIcon } from '../../icons';
import { ConfigurationCtrl } from './model';
import { formSchema } from './schema';
import './index.css';

export const Configuration: React.FC<{}> = () => {
  const vm = useController<ConfigurationCtrl>(ConfigurationCtrl);
  const { data: vmData } = vm;

  useMount(vm.init);

  if (!vmData.loaded) {
    return <span className="AnimatedEllipsis"></span>;
  }

  if (vmData.errorInfo.load) {
    return <ErrorBlock>{vmData.errorInfo.load.message}</ErrorBlock>;
  }

  return (
    <div className="configuration">
      <Caption
        extra={<Extra vm={vm} />}
        description={
          <ErrorBlock>
            {vmData.errorInfo.mutate?.message || vmData.errorInfo.code?.message}
          </ErrorBlock>
        }
      >
        {vmData.file_path || 'Configuration'}
      </Caption>
      <Content vm={vm} />
    </div>
  );
};

function Extra({ vm }: { vm: ConfigurationCtrl }) {
  const { data: vmData, services } = vm;

  if (!vmData.config) {
    return (
      <Button
        loading={services.createConfig.loading}
        size="small"
        type="primary"
        onClick={vm.handleCreateConfig}
      >
        Create
      </Button>
    );
  }

  if (RUN_TYPE_SERVER && (!vmData.config.tls_cert || !vmData.config.tls_key)) {
    return (
      <Button
        loading={services.createTLSConfig.loading}
        size="small"
        type="primary"
        onClick={vm.handleCreateTLSConfig}
      >
        Create TLS Files
      </Button>
    );
  }

  return (
    <div className="d-flex">
      <Button
        block
        loading={services.modifyConfig.loading}
        disabled={!vmData.isFormDirty}
        size="small"
        onClick={vm.handleSaveConfig}
      >
        {vmData.isSaveSuccess && !vmData.isFormDirty ? 'Saved!' : 'Save'}
      </Button>
      <Button
        className="ml-2"
        size="small"
        selected={vmData.isShowCode}
        onClick={vm.handleShowCodeClick}
      >
        <CodeIcon />
      </Button>
    </div>
  );
}

function Content({ vm }: { vm: ConfigurationCtrl }) {
  const { data: vmData } = vm;

  if (!vmData.config) {
    return <Empty />;
  }

  if (vmData.isShowCode) {
    return (
      <TextArea
        style={{ height: '65vh', fontFamily: 'monospace' }}
        value={vmData.configString}
        onChange={vm.handleConfigChange}
      />
    );
  }

  return (
    <div className="configuration-form form">
      <details className="details-overlay mb-2" open>
        <summary aria-haspopup="true">
          <strong>Basic</strong>
        </summary>
        <FormBuilder
          schema={formSchema.basic}
          data={vmData.config}
          onChange={vm.handleItemChange}
        />
      </details>

      <details className="details-overlay">
        <summary aria-haspopup="true">
          <strong>Advanced</strong>
        </summary>
        <FormBuilder
          schema={formSchema.advanced}
          data={vmData.config}
          onChange={vm.handleItemChange}
        />
      </details>
    </div>
  );
}

function Empty() {
  return (
    <div className="blankslate">
      <h3 className="blankslate-heading">You don't seem to have configuration.</h3>
      <p>Please create one before start service.</p>
      <div className="blankslate-action">
        <button
          className="btn-link"
          type="button"
          onClick={() => window.open('https://github.com/micooz/bp', '_blank')}
        >
          Learn more
        </button>
      </div>
    </div>
  );
}
