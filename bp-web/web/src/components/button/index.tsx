import { classnames } from '../../utils';
import { BaseProps } from '../common';

interface ButtonProps extends BaseProps {
  block?: boolean;
  disabled?: boolean;
  selected?: boolean;
  loading?: boolean;
  size?: 'small' | 'large';
  type?: 'primary' | 'danger' | 'outline';
  onClick?: () => void;
}

export const Button: React.FC<ButtonProps> = (props) => {
  const {
    className,
    children,
    block,
    disabled,
    selected,
    loading,
    size,
    type,
    onClick,
  } = props;

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
    // eslint-disable-next-line jsx-a11y/role-supports-aria-props
    <button
      className={classnames('btn', btnType, btnSize, block && 'btn-block', className)}
      type="button"
      aria-selected={selected}
      aria-disabled={isDisabled}
      onClick={isDisabled ? undefined : onClick}
    >
      {children}
      {loading && <span className="AnimatedEllipsis"></span>}
    </button>
  );
};
