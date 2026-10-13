import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Card from "../Components/Card";
import CardItem from "../Components/Card-item";

export default function Overview() {
  const [applications, setApplications] = useState([]);
  const [error, setError] = useState("");

  useEffect(() => {
    invoke("get_applied_jobs")
      .then(setApplications)
      .catch((err) => setError(String(err)));
  }, []);

  return (
    <section className="dashboard overview-page">
      <h1>Overview</h1>
      <Card>
        {error ? <p role="alert">{error}</p> : applications.length === 0 ? (
          <p className="empty-note">No history of applied jobs</p>
        ) : (
          [...applications].reverse().map((application) => (
            <CardItem key={application.id} {...application} />
          ))
        )}
      </Card>
    </section>
  );
}
