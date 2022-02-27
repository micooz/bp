export interface InputProps {
  name: string;
  description?: string;
  value: string;
  numeric?: boolean;
  min?: number;
  max?: number;
  placeholder?: string;
  onChange?: (value: string | number | undefined) => void;
}

export const Input: React.FC<InputProps> = (props) => {
  const {
    name,
    value,
    numeric,
    min,
    max,
    placeholder,
    onChange,
  } = props;

  function handleChange(e: any) {
    if (!onChange) {
      return;
    }
    const val = e.target.value;
    if (numeric) {
      onChange(val ? +val : undefined);
    } else {
      onChange(val);
    }
  }

  return (
    <input
      className="form-control input-block"
      type={numeric ? 'number' : 'text'}
      min={min}
      max={max}
      step="1"
      placeholder={placeholder}
      id={name}
      value={value || ''}
      onChange={handleChange}
    />
  );
};
