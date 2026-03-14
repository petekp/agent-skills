// Data processing module — extracted from core.js (chunk 3)

function processCSV(csvString) {
  const lines = csvString.trim().split("\n");
  const headers = lines[0].split(",");
  return lines.slice(1).map((line) => {
    const values = line.split(",");
    const obj = {};
    headers.forEach((h, i) => (obj[h.trim()] = values[i]?.trim()));
    return obj;
  });
}

function aggregateByField(data, field) {
  const groups = {};
  data.forEach((item) => {
    const key = item[field] || "unknown";
    if (!groups[key]) groups[key] = [];
    groups[key].push(item);
  });
  return groups;
}

function generateReport(data, title) {
  const summary = {
    title,
    totalRecords: data.length,
    generatedAt: new Date().toISOString(),
    fields: data.length > 0 ? Object.keys(data[0]) : [],
  };
  return summary;
}

module.exports = {
  processCSV,
  aggregateByField,
  generateReport,
};
