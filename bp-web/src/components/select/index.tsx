export interface SelectOption {
  label: string;
  value: string;
}

export interface SelectProps {
  name: string;
  value: string;
  options: SelectOption[];
  onChange?: (value: string) => void;
}

export const Select: React.FC<SelectProps> = (props: SelectProps) => {
  const {
    name,
    value,
    options,
    onChange,
  } = props;

  return (
    <select
      className="form-select"
      aria-label={name}
      value={value}
      onChange={(e) => onChange?.(e.target.value)}
    >
      {options.map(item => (
        <option key={item.label} value={item.value}>
          {item.label}
        </option>
      ))}
    </select>
  );
};
