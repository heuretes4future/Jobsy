export default function Toggle({ label }) {
  return (
    <label className="setting-row">
      <span>{label}</span>

      <input className="toggle-input" type="checkbox" />

      <span className="toggle">
        <span className="toggle-circle" />
      </span>
    </label>
  );
}