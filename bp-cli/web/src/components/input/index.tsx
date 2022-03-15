import { useState } from 'react';
import { EyeIcon, EyeClosedIcon } from '../../icons';
import './index.css';

export interface InputProps {
  name: string;
  type: 'text' | 'number' | 'password';
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
    type,
    value,
    min,
    max,
    placeholder,
    onChange,
  } = props;

  const [realType, setRealType] = useState(type);

  const extraProps = {
    'text': {},
    'password': {},
    'number': {
      min,
      max,
      step: '1',
    },
  }[type];

  function handleChange(e: any) {
    if (!onChange) {
      return;
    }
    const val = e.target.value;
    if (type === 'number') {
      onChange(val ? +val : undefined);
    } else {
      onChange(val);
    }
  }

  function onExtraClick(args: any) {
    if (type === 'password') {
      const on = args as boolean;
      setRealType(on ? 'text' : 'password');
    }
  }

  return (
    <div className={`input input-${type}`}>
      <input
        className="form-control input-block"
        type={realType}
        {...extraProps}
        placeholder={placeholder}
        id={name}
        value={value || ''}
        onChange={handleChange}
      />
      <InputExtra
        type={type}
        onClick={onExtraClick}
      />
    </div>
  );
};

interface InputExtraProps {
  type: InputProps['type'];
  onClick?: (args?: any) => void;
}

const InputExtra: React.FC<InputExtraProps> = (props) => {
  const { type, onClick } = props;
  const [passOn, setPassOn] = useState(false);

  if (type === 'password') {
    const handleClick = () => {
      const on = !passOn;
      setPassOn(on);
      onClick?.(on);
    };

    return (
      <div className="input-extra" onClick={handleClick}>
        {passOn ? <EyeIcon /> : <EyeClosedIcon />}
      </div>
    );
  }

  return null;
};
