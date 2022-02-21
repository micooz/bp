import { useController } from 'bizify';
import { Button, Caption, InputItem } from '../../components';
import { useMount } from '../../hooks';
import { ConfigurationCtrl } from './ctrl';

export const Configuration: React.FC<{}> = () => {
  const vm = useController<ConfigurationCtrl>(ConfigurationCtrl);
  const { data, services } = vm;
  const { loaded, config } = data;

  useMount(vm.init);

  if (!loaded) {
    return null;
  }

  return (
    <div className="configuration">
      <Caption
        extra={!config && <Button loading={services.create.loading} onClick={vm.create}>Create</Button>}
      >
        Configuration
      </Caption>
      {config ? <Content vm={vm} /> : <Empty />}
    </div>
  );
};

function Content({ vm }: { vm: ConfigurationCtrl }) {
  const config = vm.data.config!;

  return (
    <div className="form pl-2">
      <InputItem
        name="bind"
        placeholder="127.0.0.1:1080"
        description="Local service bind address [default: 127.0.0.1:1080]"
        value={config.bind}
        onChange={(v) => { }}
      />
      <InputItem
        name="with_basic_auth"
        placeholder="user:pass"
        value={config.with_basic_auth}
        onChange={(v) => { }}
      />
      TODO
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
