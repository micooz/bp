import { Checkbox, CheckboxProps } from '../../checkbox';
import { FormItemWrapper, FormItemWrapperProps } from '../form-item-wrapper';

interface FormCheckboxProps extends CheckboxProps, FormItemWrapperProps { }

export const FormCheckbox: React.FC<FormCheckboxProps> = (props) => {
  return (
    <FormItemWrapper {...props}>
      <Checkbox {...props} />
    </FormItemWrapper>
  );
};
