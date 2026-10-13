import { useState } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

{/* Gets the PDF specific URL */}
async function pdfGetURL(){
  const selectedPath = await openDialog({
    multiple: false,
    directory: false,
    filters: [
      {
        name: "PDF",
        extensions: ["pdf"],
      },
    ],
  });
  
  return selectedPath; 
}

{/* (Main function) returns 2 arguments: PDF conversion to string & rust function call*/}
export function usePdfLoad() {
  const [pdfText, setPdfText] = useState("");

  async function pdfRunLoad() {
    try {
      const filePath = await pdfGetURL();
      if (!filePath) return; // User closed window
      
      const result = await invoke('pdf_to_string', { url: filePath });
      
      console.log("Successfully unpacked from Ok:", result);
      
      setPdfText(result.text);
      return result.profile_id;
      
    } catch (error) {
      console.error("Unpacked error message from Err:", error);
      throw error;
    }
  }

  return { pdfText, pdfRunLoad };
}

/* Sorting */

function normalizeText(text) {
  return String(text ?? "")
    .toLowerCase()
    .replace(/[^\p{L}\p{N}+#.\s-]/gu, " ")
    .replace(/\s+/g, " ")
    .trim();
}

function containsKeyword(text, keyword) {
  const normalizedText = normalizeText(text);
  const normalizedKeyword = normalizeText(keyword);

  if (!normalizedKeyword) return false;

  // Multi-word skills such as "Machine learning"
  if (normalizedKeyword.includes(" ")) {
    return normalizedText.includes(normalizedKeyword);
  }

  // Avoid bad matches such as "Git" matching "digital"
  const escaped = normalizedKeyword.replace(
    /[.*+?^${}()|[\]\\]/g,
    "\\$&"
  );

  const regex = new RegExp(
    `(^|[^\\p{L}\\p{N}])${escaped}([^\\p{L}\\p{N}]|$)`,
    "iu"
  );

  return regex.test(normalizedText);
}

export function getJobRelevance(job, keywords) {
  // Add/remove fields here depending on your job object
  const jobText = [
    job.title,
    job.description,
    job.employer_name,
    job.city,
    job.employment_type,
    job.skills,
    job.requirements,
  ]
    .flat()
    .filter(Boolean)
    .join(" ");

  const matchedKeywords = keywords.filter((keyword) =>
    containsKeyword(jobText, keyword)
  );

  return {
    score: matchedKeywords.length,
    matchedKeywords,
    relevant: matchedKeywords.length > 0,
  };
}

export function sortJobsByRelevance(jobs, keywords) {
  return jobs
    .map((job, index) => ({
      job,
      index,
      relevance: getJobRelevance(job, keywords),
    }))
    .sort((a, b) => {
      // Higher number of matches first
      if (b.relevance.score !== a.relevance.score) {
        return b.relevance.score - a.relevance.score;
      }

      // Keep original order when scores are identical
      return a.index - b.index;
    })
    .map(({ job }) => job);
}


export function sortJobsByDate(jobs, order = "new") {
  return [...jobs].sort((a, b) => {
    const dateA = getJobDate(a);
    const dateB = getJobDate(b);

    // Jobs without a date always go last
    if (!dateA && !dateB) return 0;
    if (!dateA) return 1;
    if (!dateB) return -1;

    if (order === "old") {
      return dateA - dateB;
    }

    return dateB - dateA;
  });
}

function getJobDate(job) {
  const rawDate =
    job.publication_date ||
    job.published_at ||
    job.posted_at ||
    job.created_at ||
    job.date;

  if (!rawDate) {
    return null;
  }

  const parsedDate = new Date(rawDate);

  if (Number.isNaN(parsedDate.getTime())) {
    return null;
  }

  return parsedDate;
}