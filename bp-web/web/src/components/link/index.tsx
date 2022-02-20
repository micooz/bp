interface LinkProps {
  to: string;
}

export const Link: React.FC<LinkProps> = (props) => {
  const { to, children } = props;
  return <a href={to} className="Header-link" target="_blank" rel="noreferrer">{children}</a>;
};
