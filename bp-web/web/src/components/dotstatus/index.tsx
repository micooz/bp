import { BaseProps } from '../common';
import { DotFillIcon } from '../../icons';
import { classnames } from '../../utils';

interface DotStatusProps extends BaseProps {
  status: 'on' | 'off';
}

export const DotStatus: React.FC<DotStatusProps> = (props) => {
  const { className, status, children } = props;

  const color = {
    'on': 'color-fg-success',
    'off': 'color-fg-muted',
  }[status];

  return (
    <span className={classnames(className, 'd-flex flex-items-center')}>
      <DotFillIcon className={color} /><span>&nbsp;{children}</span>
    </span>
  );
};
