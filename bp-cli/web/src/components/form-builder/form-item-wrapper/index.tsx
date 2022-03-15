export interface FormItemWrapperProps {
  name: string;
  description?: string;
  required?: boolean;
}

export const FormItemWrapper: React.FC<FormItemWrapperProps> = (props) => {
  const {
    name,
    description,
    required,
    children,
  } = props;

  return (
    <div className="form-group">
      <div className="form-group-header">
        <label htmlFor={name}>
          {required && <span className="color-fg-danger">* </span>}
          {name}
          {description && <p className="h4 color-fg-subtle text-light text-small">{description}</p>}
        </label>
      </div>
      <div className="form-group-body">
        {children}
      </div>
    </div>
  );
};
