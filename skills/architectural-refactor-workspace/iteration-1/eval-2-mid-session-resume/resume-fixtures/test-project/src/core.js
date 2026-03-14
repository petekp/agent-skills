// Core module — auth extracted to auth.js (chunk 1 complete)
const auth = require("./auth");

const users = [];
const emailQueue = [];

// --- User management ---
function createUser(name, email, password) {
  const existing = users.find((u) => u.email === email);
  if (existing) throw new Error("Email already exists");
  const user = {
    id: users.length + 1,
    name,
    email,
    passwordHash: auth.hashPassword(password),
    createdAt: Date.now(),
    role: "user",
  };
  users.push(user);
  queueEmail(email, "Welcome!", `Hello ${name}, welcome to the platform!`);
  return user;
}

function findUser(id) {
  return users.find((u) => u.id === id) || null;
}

function findUserByEmail(email) {
  return users.find((u) => u.email === email) || null;
}

function updateUserRole(userId, role) {
  const user = findUser(userId);
  if (!user) throw new Error("User not found");
  user.role = role;
  return user;
}

function listUsers(filters = {}) {
  let result = [...users];
  if (filters.role) result = result.filter((u) => u.role === filters.role);
  if (filters.search)
    result = result.filter(
      (u) =>
        u.name.includes(filters.search) || u.email.includes(filters.search)
    );
  return result;
}

// --- Data processing ---
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

// --- Email notifications ---
function queueEmail(to, subject, body) {
  emailQueue.push({ to, subject, body, queuedAt: Date.now(), sent: false });
}

function processEmailQueue() {
  const pending = emailQueue.filter((e) => !e.sent);
  pending.forEach((email) => {
    console.log(`Sending email to ${email.to}: ${email.subject}`);
    email.sent = true;
    email.sentAt = Date.now();
  });
  return pending.length;
}

function getEmailHistory(userEmail) {
  return emailQueue.filter((e) => e.to === userEmail);
}

module.exports = {
  // Re-export auth for backwards compatibility
  ...auth,
  createUser,
  findUser,
  findUserByEmail,
  updateUserRole,
  listUsers,
  processCSV,
  aggregateByField,
  generateReport,
  queueEmail,
  processEmailQueue,
  getEmailHistory,
  _users: users,
  _sessions: auth._sessions,
  _emailQueue: emailQueue,
};
