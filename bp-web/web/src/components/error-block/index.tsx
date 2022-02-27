import { ErrorInfo } from '../../typings';
import { classnames } from '../../utils';
import { StopIcon } from '../../icons';
import { BaseProps } from '../common';
import './index.css';

interface ErrorBlockProps extends BaseProps {
  errorInfo: ErrorInfo | null;
}

export const ErrorBlock: React.FC<ErrorBlockProps> = (props) => {
  const { className, errorInfo } = props;
  if (!errorInfo) {
    return null;
  }
  return (
    <div className={classnames(
      'ErrorBlock',
      className,
      'p-2 mt-2 rounded color-bg-danger-emphasis d-flex'
    )}>
      <StopIcon className="mt-1 mr-2" />
      <span>{errorInfo.message}</span>
    </div>
  );
};
