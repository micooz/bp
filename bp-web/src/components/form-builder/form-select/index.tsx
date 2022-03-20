import { Select, SelectProps } from '../../select';
import { FormItemWrapper, FormItemWrapperProps } from '../form-item-wrapper';

interface FormSelectProps extends SelectProps, FormItemWrapperProps { }

export const FormSelect: React.FC<FormSelectProps> = (props) => {
  return (
    <FormItemWrapper {...props}>
      <Select {...props} />
    </FormItemWrapper>
  );
};
