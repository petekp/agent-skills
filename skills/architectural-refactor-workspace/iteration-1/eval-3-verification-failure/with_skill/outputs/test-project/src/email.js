// Email module: handles email notifications and queue

const emailQueue = [];

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
  queueEmail,
  processEmailQueue,
  getEmailHistory,
  _emailQueue: emailQueue,
};
