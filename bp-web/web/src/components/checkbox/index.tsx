import { BaseProps } from '../common';

export interface CheckboxProps extends BaseProps {
  name: string;
  checked: boolean;
  placeholder?: string;
  onChange?: (checked: boolean) => void;
}

export const Checkbox: React.FC<CheckboxProps> = (props) => {
  const {
    name,
    checked,
    placeholder,
    className,
    children = 'Enable',
    onChange,
  } = props;

  return (
    <label className={`d-flex flex-items-center ${className}`} style={{ fontWeight: 'normal' }}>
      <input
        type="checkbox"
        placeholder={placeholder}
        id={name}
        checked={checked}
        onChange={() => onChange?.(!checked)}
      />
      &nbsp;{children}
    </label>
  );
};
