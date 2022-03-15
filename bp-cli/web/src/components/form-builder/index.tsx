import { FormCheckbox } from './form-checkbox';
import { FormInput } from './form-input';
import { FormSelect } from './form-select';

export interface FormItem {
  name: string;
  key: string;
  type: 'text' | 'password' | 'boolean' | 'number' | 'select';
  required?: boolean;
  required_if?: string | string[];
  placeholder?: string;
  description?: string;
  options?: { label: string, value: any }[];
  min?: number;
  max?: number;
}

interface FormBuilderProps {
  schema: FormItem[];
  data: Record<string, any>;
  onChange?: (key: string, value: any) => void;
}

export const FormBuilder: React.FC<FormBuilderProps> = (props) => {
  const { schema, data, onChange } = props;

  return (
    <div className="FormBuilder">
      {schema.map(item => (
        <FormItemComp
          key={item.key}
          data={data}
          item={item}
          onChange={(v: any) => onChange?.(item.key, v)}
        />
      ))}
    </div>
  );
};

interface FormItemProps {
  data: Record<string, any>;
  item: FormItem;
  onChange: (value: any) => void;
}

const FormItemComp: React.FC<FormItemProps> = (props) => {
  const { data, item, onChange } = props;
  const {
    name,
    key,
    type,
    required,
    required_if,
    placeholder,
    description,
    options,
  } = item;

  let isRequired = required;

  if (required_if) {
    const keys = Array.isArray(required_if) ? required_if : [required_if];
    if (keys.some(k => data[k])) {
      isRequired = true;
    }
  }

  switch (type) {
    case 'text':
    case 'number':
    case 'password':
      return (
        <FormInput
          name={name}
          type={type}
          placeholder={placeholder}
          description={description}
          required={isRequired}
          value={data[key]}
          onChange={onChange}
        />
      );
    case 'boolean':
      return (
        <FormCheckbox
          name={name}
          placeholder={placeholder}
          description={description}
          required={isRequired}
          checked={data[key]}
          onChange={onChange}
        />
      );
    case 'select':
      return (
        <FormSelect
          name={name}
          description={description}
          required={isRequired}
          value={data[key]}
          options={options || []}
          onChange={onChange}
        />
      );
    default:
      break;
  }

  return null;
};
