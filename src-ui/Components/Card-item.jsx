export default function CardItem({ company, role, date, summary }) {
  return (
    <article className="card-item">
      <div className="card-item-header">
        <h3>{company}</h3>
        <time dateTime={date}>{new Date(date).toLocaleString()}</time>
      </div>
      <p className="card-item-role">{role}</p>
      <details className="card-item-details">
        <summary>Summary</summary>
        <p className="card-item-summary">{summary || "No description available."}</p>
      </details>
    </article>
  );
}
