interface InputItemProps {
  name: string;
  title?: string;
  description?: string;
  value: string;
  defaultValue?: string;
  placeholder?: string;
  onChange?: (value: string) => void;
}

export const InputItem: React.FC<InputItemProps> = (props) => {
  const { title, description, name, value, defaultValue, placeholder, onChange } = props;
  const val = value || defaultValue || '';

  return (
    <div className="form-group">
      <div className="form-group-header">
        <label htmlFor={name}>
          {title || name}
          {description && <p className="h4 color-fg-subtle text-light text-small">{description}</p>}
        </label>
      </div>
      <div className="form-group-body">
        <input
          className="form-control"
          type="text"
          placeholder={placeholder}
          id={name}
          value={value || defaultValue}
          onChange={() => onChange?.(val)}
        />
      </div>
    </div>
  );
};
