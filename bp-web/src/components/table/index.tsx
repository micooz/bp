interface TableProps {
  rows: React.ReactNode[][];
}

export const Table: React.FC<TableProps> = (props) => {
  const { rows } = props;

  return (
    <div className="table">
      {rows.map((row, rowIndex) => (
        <div key={rowIndex} className="d-table width-full">
          <div className="d-table-cell p-1 pl-2 no-wrap pr-2 text-semibold" style={{ width: "35vw" }}>{row[0]}</div>
          {row.slice(1).map((item, itemIndex) => (
            <div key={itemIndex} className="d-table-cell p-1 color-fg-subtle">{item}</div>
          ))}
        </div>
      ))}
    </div>
  );
};
