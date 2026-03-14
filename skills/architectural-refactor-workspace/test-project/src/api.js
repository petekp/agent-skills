// API layer that depends heavily on the god module
const core = require("./core");

function handleLogin(email, password) {
  const user = core.findUserByEmail(email);
  if (!user) return { error: "Invalid credentials" };
  if (!core.verifyPassword(password, user.passwordHash))
    return { error: "Invalid credentials" };
  const token = core.createSession(user.id);
  return { token, user: { id: user.id, name: user.name, email: user.email } };
}

function handleRegister(name, email, password) {
  try {
    const user = core.createUser(name, email, password);
    const token = core.createSession(user.id);
    return {
      token,
      user: { id: user.id, name: user.name, email: user.email },
    };
  } catch (e) {
    return { error: e.message };
  }
}

function handleUploadCSV(token, csvString, groupByField) {
  const session = core.validateSession(token);
  if (!session) return { error: "Unauthorized" };

  const data = core.processCSV(csvString);
  const grouped = groupByField
    ? core.aggregateByField(data, groupByField)
    : null;
  const report = core.generateReport(data, "CSV Upload Report");

  // Notify user about their upload - reaching into user and email concerns
  const user = core.findUser(session.userId);
  if (user) {
    core.queueEmail(
      user.email,
      "Upload Complete",
      `Your CSV with ${data.length} records has been processed.`
    );
  }

  return { data, grouped, report };
}

function handleAdminListUsers(token, filters) {
  const session = core.validateSession(token);
  if (!session) return { error: "Unauthorized" };

  const user = core.findUser(session.userId);
  if (!user || user.role !== "admin") return { error: "Forbidden" };

  return { users: core.listUsers(filters) };
}

module.exports = { handleLogin, handleRegister, handleUploadCSV, handleAdminListUsers };
