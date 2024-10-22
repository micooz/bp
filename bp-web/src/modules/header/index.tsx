import { RUN_TYPE, VERSION } from '../../common';
import { Link } from '../../components';

export const Header: React.FC<{}> = () => {
  return (
    <div className="Header">
      <div className="Header-item">
        <Link to="https://github.com/micooz/bp">About</Link>
      </div>
      <div className="Header-item">
        <Link to="https://github.com/micooz/bp/releases">Releases</Link>
      </div>
      <div className="Header-item Header-item--full">
        <Link to="https://github.com/micooz/bp/issues">Issues</Link>
      </div>
      <div className="Header-item mr-0">
        <span>{VERSION} ({RUN_TYPE})</span>
      </div>
    </div>
  );
};
