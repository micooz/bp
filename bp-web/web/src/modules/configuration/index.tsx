import { useController } from 'bizify';
import { Button, Caption } from '../../components';
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
  return (
    <div>render config here</div>
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
