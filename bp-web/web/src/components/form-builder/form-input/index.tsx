import { Input, InputProps } from '../../input';
import { FormItemWrapper, FormItemWrapperProps } from '../form-item-wrapper';

interface FormInputProps extends InputProps, FormItemWrapperProps { }

export const FormInput: React.FC<FormInputProps> = (props) => {
  return (
    <FormItemWrapper {...props}>
      <Input {...props} />
    </FormItemWrapper>
  );
};
