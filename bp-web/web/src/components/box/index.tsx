import { classnames } from '../../utils';
import { BaseProps } from '../common';

interface BoxProps extends BaseProps {
  title?: React.ReactNode;
  extra?: React.ReactNode;
  condensed?: boolean;
}

export const Box: React.FC<BoxProps> = (props) => {
  const { title, extra, condensed, className, children } = props;
  return (
    <div className={classnames('Box', condensed && 'Box--condensed', className)}>
      <div className="Box-header d-flex flex-items-center">
        <h3 className="Box-title overflow-hidden flex-auto">
          {title}
        </h3>
        {extra}
      </div>
      {children && (
        <div className="Box-body">
          {children}
        </div>
      )}
    </div>
  );
};
