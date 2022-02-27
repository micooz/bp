import { useController } from 'bizify';
import { Button, Caption, ErrorBlock, FormBuilder } from '../../components';
import { useMount } from '../../hooks';
import { RUN_TYPE_SERVER } from '../../common';
import { ConfigurationCtrl } from './model';
import { formSchema } from './schema';
import './index.css';

export const Configuration: React.FC<{}> = () => {
  const vm = useController<ConfigurationCtrl>(ConfigurationCtrl);
  const { data: vmData } = vm;

  useMount(vm.init);

  if (!vmData.loaded) {
    return null;
  }

  if (vmData.errorInfo.load) {
    return <ErrorBlock errorInfo={vmData.errorInfo.load} />;
  }

  return (
    <div className="Configuration">
      <Caption extra={<Extra vm={vm} />} description={<ErrorBlock errorInfo={vmData.errorInfo.mutate} />}>
        Configuration
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
        loading={services.createSecurityConfig.loading}
        size="small"
        type="primary"
        onClick={vm.handleCreateCertKey}
      >
        Create TLS Files
      </Button>
    );
  }

  return (
    <Button
      block
      loading={services.modifyConfig.loading}
      disabled={!vmData.isFormDirty}
      size="small"
      onClick={vm.handleSaveConfig}
    >
      {vmData.isSaveSuccess && !vmData.isFormDirty ? 'Saved!' : 'Save'}
    </Button>
  );
}

function Content({ vm }: { vm: ConfigurationCtrl }) {
  const { data: vmData } = vm;

  if (!vmData.config) {
    return <Empty />;
  }

  return (
    <div className="Configuration-form form">
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
      <p>Pull requests help you discuss potential changes before they are merged into the base branch.</p>
      <div className="blankslate-action">
        <button className="btn-link" type="button">Learn more</button>
      </div>
    </div>
  );
}
