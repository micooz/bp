interface ButtonProps {
  loading?: boolean;
  onClick?: () => {};
}

export const Button: React.FC<ButtonProps> = (props) => {
  const { children, loading, onClick } = props;

  return (
    <button
      className="btn btn-primary btn-sm"
      type="button"
      aria-disabled={loading}
      onClick={onClick}
    >
      {children}
      {loading && <span className="AnimatedEllipsis"></span>}
    </button>
  );
};
