interface CaptionProps {
  extra?: React.ReactNode;
}

export const Caption: React.FC<CaptionProps> = (props) => {
  const { extra, children } = props;

  return (
    <div className="Subhead">
      <div className="Subhead-heading d-flex flex-justify-between flex-items-center" style={{ fontSize: "16px" }}>
        <div className="h4">{children}</div>
        {extra && <div>{extra}</div>}
      </div>
    </div>
  )
};
