import { classnames } from '../../utils';
import { XIcon } from '../../icons';

interface AlertProps {
  type: 'warn' | 'error' | 'success';
  closable?: boolean;
  onClose?: () => void;
}

export const Alert: React.FC<AlertProps> = (props) => {
  const { type, closable, onClose, children } = props;

  const flashType = {
    '': '',
    'warn': 'flash-warn',
    'error': 'flash-error',
    'success': 'flash-success',
  }[type];

  return (
    <div className={classnames('flash mt-2 mb-2', flashType)}>
      {children}
      {closable && (
        <button
          className="flash-close js-flash-close"
          type="button"
          aria-label="Close"
          onClick={onClose}
        >
          <XIcon />
        </button>
      )}
    </div>
  );
};
