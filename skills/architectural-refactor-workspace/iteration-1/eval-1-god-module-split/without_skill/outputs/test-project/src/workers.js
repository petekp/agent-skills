// Background workers
const email = require("./email");
const users = require("./users");

function runEmailWorker() {
  const sent = email.processEmailQueue();
  return { processed: sent };
}

function runUserCleanup() {
  const now = Date.now();
  const inactiveThreshold = 30 * 24 * 60 * 60 * 1000; // 30 days

  const allUsers = users.listUsers();
  const inactive = allUsers.filter(
    (u) => now - u.createdAt > inactiveThreshold
  );

  inactive.forEach((u) => {
    email.queueEmail(
      u.email,
      "We miss you!",
      `Hi ${u.name}, it's been a while!`
    );
  });

  return { notified: inactive.length };
}

module.exports = { runEmailWorker, runUserCleanup };
