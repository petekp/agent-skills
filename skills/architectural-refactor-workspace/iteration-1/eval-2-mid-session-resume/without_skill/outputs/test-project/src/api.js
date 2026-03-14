// API layer — imports from focused domain modules
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

  const parsedData = data.processCSV(csvString);
  const grouped = groupByField
    ? data.aggregateByField(parsedData, groupByField)
    : null;
  const report = data.generateReport(parsedData, "CSV Upload Report");

  // Notify user about their upload
  const user = users.findUser(session.userId);
  if (user) {
    email.queueEmail(
      user.email,
      "Upload Complete",
      `Your CSV with ${parsedData.length} records has been processed.`
    );
  }

  return { data: parsedData, grouped, report };
}

function handleAdminListUsers(token, filters) {
  const session = auth.validateSession(token);
  if (!session) return { error: "Unauthorized" };

  const user = users.findUser(session.userId);
  if (!user || user.role !== "admin") return { error: "Forbidden" };

  return { users: users.listUsers(filters) };
}

module.exports = { handleLogin, handleRegister, handleUploadCSV, handleAdminListUsers };
