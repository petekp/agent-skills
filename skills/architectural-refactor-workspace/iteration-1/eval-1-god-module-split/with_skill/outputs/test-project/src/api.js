// API layer: imports from domain modules directly
const auth = require("./auth");
const users = require("./users");
const data = require("./data");
const email = require("./email");

function handleLogin(emailAddr, password) {
  const user = users.findUserByEmail(emailAddr);
  if (!user) return { error: "Invalid credentials" };
  if (!auth.verifyPassword(password, user.passwordHash))
    return { error: "Invalid credentials" };
  const token = auth.createSession(user.id);
  return { token, user: { id: user.id, name: user.name, email: user.email } };
}

function handleRegister(name, emailAddr, password) {
  try {
    const user = users.createUser(name, emailAddr, password);
    const token = auth.createSession(user.id);
    return {
      token,
      user: { id: user.id, name: user.name, email: user.email },
    };
  } catch (e) {
    return { error: e.message };
  }
}

function handleUploadCSV(token, csvString, groupByField) {
  const session = auth.validateSession(token);
  if (!session) return { error: "Unauthorized" };

  const csvData = data.processCSV(csvString);
  const grouped = groupByField
    ? data.aggregateByField(csvData, groupByField)
    : null;
  const report = data.generateReport(csvData, "CSV Upload Report");

  // Notify user about their upload
  const user = users.findUser(session.userId);
  if (user) {
    email.queueEmail(
      user.email,
      "Upload Complete",
      `Your CSV with ${csvData.length} records has been processed.`
    );
  }

  return { data: csvData, grouped, report };
}

function handleAdminListUsers(token, filters) {
  const session = auth.validateSession(token);
  if (!session) return { error: "Unauthorized" };

  const user = users.findUser(session.userId);
  if (!user || user.role !== "admin") return { error: "Forbidden" };

  return { users: users.listUsers(filters) };
}

module.exports = { handleLogin, handleRegister, handleUploadCSV, handleAdminListUsers };
