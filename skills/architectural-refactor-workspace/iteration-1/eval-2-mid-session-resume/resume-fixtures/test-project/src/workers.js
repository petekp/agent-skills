// Background worker that also depends on the god module for everything
const core = require("./core");

function runEmailWorker() {
  const sent = core.processEmailQueue();
  return { processed: sent };
}

function runUserCleanup() {
  // Example of feature envy: reaching into core's internals
  const now = Date.now();
  const inactiveThreshold = 30 * 24 * 60 * 60 * 1000; // 30 days

  const allUsers = core.listUsers();
  const inactive = allUsers.filter(
    (u) => now - u.createdAt > inactiveThreshold
  );

  inactive.forEach((u) => {
    core.queueEmail(
      u.email,
      "We miss you!",
      `Hi ${u.name}, it's been a while!`
    );
  });

  return { notified: inactive.length };
}

module.exports = { runEmailWorker, runUserCleanup };
