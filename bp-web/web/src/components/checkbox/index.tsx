export interface CheckboxProps {
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
    onChange,
  } = props;

  return (
    <label className="d-flex flex-items-center" style={{ fontWeight: 'normal' }}>
      <input
        type="checkbox"
        placeholder={placeholder}
        id={name}
        checked={checked}
        onChange={() => onChange?.(!checked)}
      />
      &nbsp;Enable
    </label>
  );
};
