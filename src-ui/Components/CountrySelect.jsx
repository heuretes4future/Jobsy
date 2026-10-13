import { COUNTRIES } from "../Utils/Country_index/countries.js"

export default function CountrySelect({ value, onChange }) {
  
  return (
    <select
      value={value}
      onChange={(event) => onChange(event.target.value)}
      style={{
        width: "50%",
        padding: "10px 12px",
        border: "1px solid #ccc",
        borderRadius: "10px",
        backgroundColor: "white",
        fontSize: "16px",
              }}
    >
      <option value="">Select a country</option>

      {COUNTRIES.map((country) => (
        <option key={country.value} value={country.value}>
          {country.title}
        </option>
      ))}
    </select>
  );
}