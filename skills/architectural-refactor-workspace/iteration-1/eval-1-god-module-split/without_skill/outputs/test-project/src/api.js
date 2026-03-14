// API layer
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

  const parsed = data.processCSV(csvString);
  const grouped = groupByField
    ? data.aggregateByField(parsed, groupByField)
    : null;
  const report = data.generateReport(parsed, "CSV Upload Report");

  const user = users.findUser(session.userId);
  if (user) {
    email.queueEmail(
      user.email,
      "Upload Complete",
      `Your CSV with ${parsed.length} records has been processed.`
    );
  }

  return { data: parsed, grouped, report };
}

function handleAdminListUsers(token, filters) {
  const session = auth.validateSession(token);
  if (!session) return { error: "Unauthorized" };

  const user = users.findUser(session.userId);
  if (!user || user.role !== "admin") return { error: "Forbidden" };

  return { users: users.listUsers(filters) };
}

module.exports = { handleLogin, handleRegister, handleUploadCSV, handleAdminListUsers };
