// Simple test runner — imports from individual modules instead of core
const auth = require("../src/auth");
const users = require("../src/users");
const data = require("../src/data");
const email = require("../src/email");
const api = require("../src/api");
const workers = require("../src/workers");

let passed = 0;
let failed = 0;

function test(name, fn) {
  try {
    fn();
    passed++;
    console.log(`  PASS: ${name}`);
  } catch (e) {
    failed++;
    console.log(`  FAIL: ${name} - ${e.message}`);
  }
}

function assert(condition, msg) {
  if (!condition) throw new Error(msg || "Assertion failed");
}

// Reset state between tests
function resetState() {
  users._users.length = 0;
  Object.keys(auth._sessions).forEach((k) => delete auth._sessions[k]);
  email._emailQueue.length = 0;
}

console.log("Running tests...\n");

// Auth tests
resetState();
test("hashPassword returns base64", () => {
  const hash = auth.hashPassword("test123");
  assert(hash === Buffer.from("test123").toString("base64"));
});

test("verifyPassword works correctly", () => {
  const hash = auth.hashPassword("mypass");
  assert(auth.verifyPassword("mypass", hash));
  assert(!auth.verifyPassword("wrong", hash));
});

test("createSession and validateSession", () => {
  const token = auth.createSession(1);
  assert(token, "Should return a token");
  const session = auth.validateSession(token);
  assert(session, "Should validate the token");
  assert(session.userId === 1, "Should have correct userId");
});

test("validateSession rejects invalid token", () => {
  assert(auth.validateSession("fake") === null);
});

// User tests
resetState();
test("createUser creates a user", () => {
  const user = users.createUser("Alice", "alice@test.com", "pass123");
  assert(user.id === 1);
  assert(user.name === "Alice");
  assert(user.email === "alice@test.com");
});

test("createUser rejects duplicate email", () => {
  try {
    users.createUser("Bob", "alice@test.com", "pass456");
    assert(false, "Should have thrown");
  } catch (e) {
    assert(e.message === "Email already exists");
  }
});

test("findUser and findUserByEmail", () => {
  const byId = users.findUser(1);
  assert(byId && byId.name === "Alice");
  const byEmail = users.findUserByEmail("alice@test.com");
  assert(byEmail && byEmail.name === "Alice");
  assert(users.findUser(999) === null);
});

test("listUsers with filters", () => {
  users.createUser("Bob", "bob@test.com", "pass");
  users.updateUserRole(1, "admin");
  const admins = users.listUsers({ role: "admin" });
  assert(admins.length === 1 && admins[0].name === "Alice");
  const searched = users.listUsers({ search: "bob" });
  assert(searched.length === 1 && searched[0].name === "Bob");
});

// API tests
resetState();
test("handleRegister and handleLogin", () => {
  const reg = api.handleRegister("Eve", "eve@test.com", "secret");
  assert(!reg.error, "Register should succeed");
  assert(reg.token, "Should get a token");

  const login = api.handleLogin("eve@test.com", "secret");
  assert(!login.error, "Login should succeed");
  assert(login.token, "Should get a token");
});

test("handleLogin rejects bad credentials", () => {
  const result = api.handleLogin("eve@test.com", "wrong");
  assert(result.error === "Invalid credentials");
});

test("handleUploadCSV processes data", () => {
  const reg = api.handleRegister("Uploader", "up@test.com", "pass");
  const csv = "name,age\nAlice,30\nBob,25";
  const result = api.handleUploadCSV(reg.token, csv);
  assert(!result.error);
  assert(result.data.length === 2);
  assert(result.report.totalRecords === 2);
});

test("handleUploadCSV rejects invalid token", () => {
  const result = api.handleUploadCSV("bad-token", "a,b\n1,2");
  assert(result.error === "Unauthorized");
});

// Data processing tests
test("processCSV parses correctly", () => {
  const csvData = data.processCSV("x,y\n1,2\n3,4");
  assert(csvData.length === 2);
  assert(csvData[0].x === "1" && csvData[0].y === "2");
});

test("aggregateByField groups correctly", () => {
  const items = [
    { type: "a", val: 1 },
    { type: "b", val: 2 },
    { type: "a", val: 3 },
  ];
  const grouped = data.aggregateByField(items, "type");
  assert(grouped.a.length === 2);
  assert(grouped.b.length === 1);
});

// Email tests
resetState();
test("queueEmail and processEmailQueue", () => {
  email.queueEmail("test@test.com", "Hi", "Body");
  assert(email._emailQueue.length === 1);
  const sent = email.processEmailQueue();
  assert(sent === 1);
  assert(email._emailQueue[0].sent === true);
});

test("getEmailHistory returns correct emails", () => {
  email.queueEmail("a@test.com", "S1", "B1");
  email.queueEmail("b@test.com", "S2", "B2");
  email.queueEmail("a@test.com", "S3", "B3");
  const history = email.getEmailHistory("a@test.com");
  assert(history.length === 2);
});

// Worker tests
resetState();
test("runEmailWorker processes queue", () => {
  email.queueEmail("w@test.com", "Test", "Body");
  const result = workers.runEmailWorker();
  assert(result.processed === 1);
});

console.log(`\nResults: ${passed} passed, ${failed} failed`);
if (failed > 0) process.exit(1);
