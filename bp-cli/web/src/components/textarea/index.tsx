import React from 'react';
import { classnames } from '../../utils';
import { BaseProps } from '../common';

interface TextAreaProps extends BaseProps {
  value: string;
  readonly?: boolean;
  onChange?: (v: string) => void;
}

export const TextArea = React.forwardRef<HTMLTextAreaElement, TextAreaProps>((props, ref) => {
  const { className, style, value, readonly, onChange } = props;
  return (
    <textarea
      ref={ref}
      className={classnames('form-control', className)}
      spellCheck={false}
      style={{
        width: '100%',
        ...style,
      }}
      value={value}
      onChange={e => onChange?.(e.target.value)}
      readOnly={readonly}
    />
  );
});
