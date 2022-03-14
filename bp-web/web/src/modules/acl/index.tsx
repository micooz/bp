import { useController } from 'bizify';
import { Caption, Button, ErrorBlock, TextArea } from '../../components';
import { useMount } from '../../hooks';
import { AclCtrl } from './model';

export const Acl: React.FC<{}> = () => {
  const vm = useController<AclCtrl>(AclCtrl);
  const { data: vmData, services } = vm;

  useMount(vm.init);

  return (
    <div className="acl">
      <ErrorBlock className="mb-3">{services.query.error?.message}</ErrorBlock>
      <Caption
        extra={<Button size="small" loading={services.modify.loading} onClick={vm.handleSave}>Save</Button>}
        description={<ErrorBlock>{services.modify.error?.message}</ErrorBlock>}
      >
        {vmData.file_path || 'ACL'}
      </Caption>
      <TextArea
        className="logs-textarea"
        style={{ height: '65vh' }}
        value={vmData.content}
        onChange={vm.handleInputChange}
      />
    </div>
  );
};
