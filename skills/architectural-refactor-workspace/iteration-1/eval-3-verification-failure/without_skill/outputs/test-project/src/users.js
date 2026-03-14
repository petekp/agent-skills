// User management module: CRUD operations for users

const { hashPassword } = require("./auth");
const { queueEmail } = require("./email");

const users = [];

function createUser(name, email, password) {
  const existing = users.find((u) => u.email === email);
  if (existing) throw new Error("Email already exists");
  const user = {
    id: users.length + 1,
    name,
    email,
    passwordHash: hashPassword(password),
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

module.exports = {
  createUser,
  findUser,
  findUserByEmail,
  updateUserRole,
  listUsers,
  _users: users,
};
