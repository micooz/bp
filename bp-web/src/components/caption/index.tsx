import { classnames } from '../../utils';
import { BaseProps } from '../common';
import './index.css';

interface CaptionProps extends BaseProps {
  description?: React.ReactNode;
  extra?: React.ReactNode;
}

export const Caption: React.FC<CaptionProps> = (props) => {
  const { description, extra, children, className, style } = props;

  return (
    <div className={classnames('Caption Subhead', className)} style={style}>
      <div
        className="Caption-heading Subhead-heading d-flex flex-justify-between flex-items-center"
        style={{ fontSize: "16px" }}
      >
        <div className="h4">{children}</div>
        {extra && <div className="Caption-extra">{extra}</div>}
      </div>
      <div className="Subhead-description">{description}</div>
    </div>
  );
};
