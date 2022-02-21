import { Caption } from '../../components';
import { useModel} from './model';

export const Security: React.FC<{}> = () => {
  const { data } = useModel();

  return (
    <div className="security">
      <Caption
        extra={<button className="btn btn-primary btn-sm" type="button">Create</button>}
      >
        Security
      </Caption>
      <div className="d-table width-full">
        <div className="d-table-cell p-1 pl-2 no-wrap pr-2 text-semibold" style={{ width: "30vw" }}>TLS Certificate</div>
        <div className="d-table-cell p-1 color-fg-subtle">None</div>
      </div>
      <div className="d-table width-full">
        <div className="d-table-cell p-1 pl-2 no-wrap pr-2 text-semibold" style={{ width: "30vw" }}>TLS Private Key</div>
        <div className="d-table-cell p-1 color-fg-subtle">None</div>
      </div>
    </div>
  );
};
