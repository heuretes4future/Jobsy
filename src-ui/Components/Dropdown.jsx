export default function Dropdown({ id, value, onChange, options, placeholder = "Select an option" }) {
  return (
    <select id={id} className="filter-select" value={value} onChange={(event) => onChange(event.target.value)}>
      <option value="">{placeholder}</option>
      {options.map((option) => (
        <option key={option.value} value={option.value}>{option.title}</option>
      ))}
    </select>
  );
}
