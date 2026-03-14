// Simple test runner
const core = require("../src/core");
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
  core._users.length = 0;
  Object.keys(core._sessions).forEach((k) => delete core._sessions[k]);
  core._emailQueue.length = 0;
}

console.log("Running tests...\n");

// Auth tests
resetState();
test("hashPassword returns base64", () => {
  const hash = core.hashPassword("test123");
  assert(hash === Buffer.from("test123").toString("base64"));
});

test("verifyPassword works correctly", () => {
  const hash = core.hashPassword("mypass");
  assert(core.verifyPassword("mypass", hash));
  assert(!core.verifyPassword("wrong", hash));
});

test("createSession and validateSession", () => {
  const token = core.createSession(1);
  assert(token, "Should return a token");
  const session = core.validateSession(token);
  assert(session, "Should validate the token");
  assert(session.userId === 1, "Should have correct userId");
});

test("validateSession rejects invalid token", () => {
  assert(core.validateSession("fake") === null);
});

// User tests
resetState();
test("createUser creates a user", () => {
  const user = core.createUser("Alice", "alice@test.com", "pass123");
  assert(user.id === 1);
  assert(user.name === "Alice");
  assert(user.email === "alice@test.com");
});

test("createUser rejects duplicate email", () => {
  try {
    core.createUser("Bob", "alice@test.com", "pass456");
    assert(false, "Should have thrown");
  } catch (e) {
    assert(e.message === "Email already exists");
  }
});

test("findUser and findUserByEmail", () => {
  const byId = core.findUser(1);
  assert(byId && byId.name === "Alice");
  const byEmail = core.findUserByEmail("alice@test.com");
  assert(byEmail && byEmail.name === "Alice");
  assert(core.findUser(999) === null);
});

test("listUsers with filters", () => {
  core.createUser("Bob", "bob@test.com", "pass");
  core.updateUserRole(1, "admin");
  const admins = core.listUsers({ role: "admin" });
  assert(admins.length === 1 && admins[0].name === "Alice");
  const searched = core.listUsers({ search: "bob" });
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
  const data = core.processCSV("x,y\n1,2\n3,4");
  assert(data.length === 2);
  assert(data[0].x === "1" && data[0].y === "2");
});

test("aggregateByField groups correctly", () => {
  const data = [
    { type: "a", val: 1 },
    { type: "b", val: 2 },
    { type: "a", val: 3 },
  ];
  const grouped = core.aggregateByField(data, "type");
  assert(grouped.a.length === 2);
  assert(grouped.b.length === 1);
});

// Email tests
resetState();
test("queueEmail and processEmailQueue", () => {
  core.queueEmail("test@test.com", "Hi", "Body");
  assert(core._emailQueue.length === 1);
  const sent = core.processEmailQueue();
  assert(sent === 1);
  assert(core._emailQueue[0].sent === true);
});

test("getEmailHistory returns correct emails", () => {
  core.queueEmail("a@test.com", "S1", "B1");
  core.queueEmail("b@test.com", "S2", "B2");
  core.queueEmail("a@test.com", "S3", "B3");
  const history = core.getEmailHistory("a@test.com");
  assert(history.length === 2);
});

// Worker tests
resetState();
test("runEmailWorker processes queue", () => {
  core.queueEmail("w@test.com", "Test", "Body");
  const result = workers.runEmailWorker();
  assert(result.processed === 1);
});

console.log(`\nResults: ${passed} passed, ${failed} failed`);
if (failed > 0) process.exit(1);
