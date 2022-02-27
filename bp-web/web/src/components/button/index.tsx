import { classnames } from '../../utils';

interface ButtonProps {
  block?: boolean;
  disabled?: boolean;
  loading?: boolean;
  size?: 'small' | 'large';
  type?: 'primary' | 'danger' | 'outline';
  onClick?: () => void;
}

export const Button: React.FC<ButtonProps> = (props) => {
  const { children, block, disabled, loading, size, type, onClick } = props;

  const btnType = {
    '': '',
    'primary': 'btn-primary',
    'danger': 'btn-danger',
    'outline': 'btn-outline',
  }[type || ''];

  const btnSize = {
    '': '',
    'small': 'btn-sm',
    'large': 'btn-large',
  }[size || ''];

  const isDisabled = loading || disabled;

  return (
    <button
      className={classnames('btn', btnType, btnSize, block && 'btn-block')}
      type="button"
      aria-disabled={isDisabled}
      onClick={isDisabled ? undefined : onClick}
    >
      {children}
      {loading && <span className="AnimatedEllipsis"></span>}
    </button>
  );
};
