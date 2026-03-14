// Facade module: re-exports from focused modules for backward compatibility
const auth = require("./auth");
const users = require("./users");
const data = require("./data");
const email = require("./email");

module.exports = {
  // Auth
  hashPassword: auth.hashPassword,
  verifyPassword: auth.verifyPassword,
  createSession: auth.createSession,
  validateSession: auth.validateSession,
  // Users
  createUser: users.createUser,
  findUser: users.findUser,
  findUserByEmail: users.findUserByEmail,
  updateUserRole: users.updateUserRole,
  listUsers: users.listUsers,
  // Data processing
  processCSV: data.processCSV,
  aggregateByField: data.aggregateByField,
  generateReport: data.generateReport,
  // Email
  queueEmail: email.queueEmail,
  processEmailQueue: email.processEmailQueue,
  getEmailHistory: email.getEmailHistory,
  // Internal state (for backward-compatible test access)
  _users: users._users,
  _sessions: auth._sessions,
  _emailQueue: email._emailQueue,
};
