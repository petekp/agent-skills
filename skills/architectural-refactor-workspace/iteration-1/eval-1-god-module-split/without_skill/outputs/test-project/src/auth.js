// Authentication: password hashing, sessions

const sessions = {};

function hashPassword(password) {
  return Buffer.from(password).toString("base64");
}

function verifyPassword(password, hash) {
  return hashPassword(password) === hash;
}

function createSession(userId) {
  const token = Math.random().toString(36).substring(2);
  sessions[token] = { userId, createdAt: Date.now() };
  return token;
}

function validateSession(token) {
  const session = sessions[token];
  if (!session) return null;
  if (Date.now() - session.createdAt > 3600000) {
    delete sessions[token];
    return null;
  }
  return session;
}

module.exports = {
  hashPassword,
  verifyPassword,
  createSession,
  validateSession,
  _sessions: sessions,
};
