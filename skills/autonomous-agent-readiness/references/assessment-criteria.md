# Assessment Criteria

Detailed scoring rubrics for each dimension of autonomous agent readiness.

## Scoring Guide

- **0**: Not present or actively harmful to autonomy
- **1**: Minimal support, significant gaps
- **2**: Adequate support, room for improvement
- **3**: Strong support, follows best practices

---

## 1. Sandbox Isolation

**What it means:** Every agent run executes in its own ephemeral, isolated, disposable environment.

### Score 0
- No containerization or isolation
- Agents run directly on shared developer machines
- No environment cleanup between runs
- Shared mutable state across runs

### Score 1
- Basic Docker support exists but not used for agent runs
- Manual environment setup required
- Some cleanup scripts but inconsistent use
- Partial isolation (e.g., virtualenv but shared filesystem)

### Score 2
- Containerized execution available
- Automated environment provisioning
- Cleanup happens but not always complete
- Network access exists but not scoped

### Score 3
- Every run gets fresh container/VM
- Writable filesystem isolated per run
- Scoped network access (allowlist-based)
- Automatic teardown after verification
- No persistent state between runs

### What to look for
- `Dockerfile`, `docker-compose.yml`
- `devcontainer.json`
- VM provisioning scripts (Vagrant, Terraform)
- CI isolation configuration
- Network policies, firewall rules

### Common patterns by stack

**Node.js:**
```dockerfile
FROM node:20-alpine
WORKDIR /workspace
COPY package*.json ./
RUN npm ci
COPY . .
```

**Python:**
```dockerfile
FROM python:3.11-slim
WORKDIR /workspace
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
```

---

## 2. Database Independence

**What it means:** Agents create their own databases inside the sandbox. No shared or external database dependencies.

### Score 0
- Hardcoded connections to external databases
- No local database option
- Shared staging/dev databases
- No migration scripts

### Score 1
- Local database possible but not default
- Migrations exist but require manual setup
- Some seed data but incomplete
- External DB used for most testing

### Score 2
- Local database setup documented
- Migrations run automatically
- Seed data available
- External DB only for integration tests

### Score 3
- Database spins up as part of environment
- Migrations run on every fresh start
- Explicit seed data for all scenarios
- Complete teardown after runs
- Zero external database dependencies for agent work

### What to look for
- `docker-compose.yml` with database services
- Migration scripts (`migrations/`, `db/migrate/`)
- Seed files (`seeds/`, `fixtures/`)
- Database initialization in CI
- Connection string configuration

### Red flags
- `.env` files with production/staging URLs
- Connection pooling to shared instances
- Missing `DATABASE_URL` override options
- No test database configuration

---

## 3. Environment Reproducibility

**What it means:** Explicit dependencies, no hidden state, deterministic setup.

### Score 0
- Dependencies not pinned
- Setup requires tribal knowledge
- Different behavior on different machines
- Hidden dependencies on system packages

### Score 1
- Some dependencies pinned
- README documents setup but incomplete
- Occasional "works on my machine" issues
- Some system dependencies documented

### Score 2
- All direct dependencies pinned
- Setup mostly automated
- CI environment matches local (mostly)
- System dependencies documented

### Score 3
- All dependencies pinned with lockfiles
- Single command setup
- CI and local environments identical
- System dependencies containerized
- Deterministic builds verified

### What to look for
- Lockfiles (`package-lock.json`, `yarn.lock`, `poetry.lock`, `Gemfile.lock`)
- Dependency pinning in requirements
- Setup scripts (`setup.sh`, `make dev`)
- `.tool-versions`, `.nvmrc`, `.python-version`
- CI configuration matching local

### Environment garbage indicators
- `node_modules/` not in `.gitignore` (or checked in)
- Global package installations required
- Manual PATH modifications
- Undocumented environment variables
- Cache directories affecting behavior

---

## 4. Session Independence

**What it means:** Agent loop decoupled from browser tabs, terminal sessions, developer machines.

### Score 0
- Requires active terminal session
- Progress lost on disconnect
- No background execution option
- Results only visible in real-time

### Score 1
- Can run in background (nohup, screen)
- Some result persistence
- No proper job management
- Manual recovery after disconnect

### Score 2
- Background job support
- Results persist to files
- Basic timeout handling
- Can reconnect to view progress

### Score 3
- Remote execution by default
- Full result persistence
- Wall-clock limits enforced
- Automatic cleanup on timeout
- Client disconnect doesn't interrupt
- Job status queryable

### What to look for
- Job queue systems (Celery, Bull, Sidekiq)
- Background worker configuration
- Result storage (S3, local files, database)
- Timeout configuration
- Process management (systemd, supervisord)

### Architecture patterns

**Good:**
```
Client → API → Job Queue → Worker → Results Storage
              ↓
         Client polls for status
```

**Bad:**
```
Client → Long-running request → Results
         (breaks on disconnect)
```

---

## 5. Outcome-Oriented Design

**What it means:** Define desired outcomes and constraints, not step-by-step procedures.

### Score 0
- All operations require explicit steps
- No concept of goals or acceptance criteria
- Agent cannot make decisions
- Micromanaged tool usage

### Score 1
- Some operations goal-based
- Partial acceptance criteria
- Limited agent autonomy
- Mix of procedural and outcome-based

### Score 2
- Most operations goal-based
- Clear acceptance criteria for common tasks
- Agent has meaningful autonomy
- Constraints documented

### Score 3
- Operations defined by outcomes
- Verifiable acceptance criteria
- Agent owns planning and execution
- Constraints enforce boundaries without dictating steps

### What to look for
- Task definitions (issues, tickets, specs)
- Test suites as acceptance criteria
- Constraint documentation
- Agent decision logs
- Outcome verification scripts

### Good vs bad task definitions

**Bad (procedural):**
```
1. Open file X
2. Find function Y
3. Add parameter Z
4. Update tests
5. Run linter
```

**Good (outcome-based):**
```
Goal: Function Y should accept parameter Z
Constraints: Must pass all tests, linter clean
Acceptance: Unit test for new parameter exists and passes
```

---

## 6. Direct Interfaces

**What it means:** Direct access to command execution, persistent files, network requests. OS primitives over abstraction layers.

### Score 0
- Heavy abstraction layers
- No CLI access
- Custom protocols for everything
- Framework-locked operations

### Score 1
- Some CLI tools available
- Abstraction layers common
- Mixed direct and indirect access
- Some operations require framework

### Score 2
- CLI-first for most operations
- Minimal abstraction
- Direct file access
- Standard protocols (HTTP, SSH)

### Score 3
- All operations CLI-accessible
- OS primitives preferred
- No unnecessary abstraction
- Composable commands
- Standard Unix patterns

### What to look for
- CLI entrypoints (`bin/`, `scripts/`)
- Makefile or similar task runners
- Direct file operations vs ORM-only
- Standard protocols vs custom
- Shell script availability

### Abstraction anti-patterns
- Custom RPC when HTTP works
- Framework-specific task runners only
- No way to run operations outside the app
- Plugins required for basic operations

---

## 7. Explicit State

**What it means:** Writable workspace directory for intermediate results, logs, partial outputs, planning artifacts.

### Score 0
- State only in memory
- No logging
- Results lost on failure
- No intermediate checkpoints

### Score 1
- Basic logging exists
- Some file output
- Inconsistent state management
- Partial checkpoint support

### Score 2
- Structured logging
- Results written to files
- Workspace directories used
- Checkpoints for long operations

### Score 3
- All state in inspectable files
- Structured logs with levels
- Intermediate results persisted
- Planning artifacts saved
- Post-run analysis possible
- Clear workspace directory structure

### What to look for
- Logging configuration
- Output directory conventions
- Checkpoint/resume support
- Artifact storage patterns
- Log aggregation setup

### Workspace structure example
```
workspace/
├── logs/
│   ├── agent.log
│   └── commands.log
├── artifacts/
│   ├── plan.json
│   └── results.json
├── intermediate/
│   └── step_1_output.json
└── final/
    └── deliverable.zip
```

---

## 8. Benchmarking

**What it means:** Measurable quality criteria, automated verification. Benchmarks exist early, not as a finishing step.

### Score 0
- No quality metrics
- Manual verification only
- No baseline comparisons
- Quality is subjective

### Score 1
- Some tests exist
- Basic pass/fail metrics
- No performance baselines
- Occasional quality checks

### Score 2
- Test coverage tracked
- Performance benchmarks exist
- Quality gates in CI
- Some automated verification

### Score 3
- Comprehensive quality metrics
- Automated benchmarks on every run
- Performance regression detection
- Quality comparisons across versions
- Representative and repeatable metrics

### What to look for
- Test coverage reports
- Performance benchmark suites
- Quality gates in CI
- Metric tracking over time
- A/B comparison infrastructure

### Benchmark types
- **Correctness:** Test suite pass rate
- **Performance:** Execution time, memory usage
- **Quality:** Output quality scores (if applicable)
- **Cost:** Token usage, API calls, compute time

---

## 9. Cost Awareness

**What it means:** Token usage provisioned, compute allocated explicitly, limits enforced by system.

### Score 0
- No cost tracking
- Unlimited resource usage
- No budget awareness
- Costs discovered after the fact

### Score 1
- Basic monitoring exists
- Some manual limits
- Cost visible but not controlled
- Post-hoc cost analysis

### Score 2
- Resource limits in place
- Usage tracking per task
- Budget alerts configured
- Some cost optimization

### Score 3
- Explicit resource provisioning
- Per-task cost tracking
- System-enforced limits
- Cost as operational input
- Automatic scaling within budget

### What to look for
- Resource limits in container configs
- Token/API usage tracking
- Budget configuration
- Cost dashboards
- Automatic cutoffs

### Resource limit examples

**Docker:**
```yaml
services:
  agent:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G
```

**Kubernetes:**
```yaml
resources:
  requests:
    memory: "2Gi"
    cpu: "1"
  limits:
    memory: "4Gi"
    cpu: "2"
```

---

## Stack-Specific Indicators

### Node.js/TypeScript
- `package-lock.json` or `yarn.lock` present
- `npm ci` used instead of `npm install`
- `.nvmrc` or `engines` field in package.json
- No global package requirements

### Python
- `requirements.txt` with pinned versions or `poetry.lock`
- `pyproject.toml` for modern projects
- `.python-version` file
- Virtual environment in setup instructions

### Ruby
- `Gemfile.lock` committed
- `.ruby-version` file
- Bundler used consistently

### Go
- `go.mod` and `go.sum` committed
- No vendor directory issues
- Clear build instructions

### Rust
- `Cargo.lock` committed (for applications)
- Clear build profile documentation

---

## Quick Assessment Checklist

Run through these questions for a rapid assessment:

1. Can I run the full test suite in a fresh container? (Sandbox)
2. Does the test suite work without any external services? (Database)
3. Is there a single command to set up the development environment? (Reproducibility)
4. Can I start a task and disconnect without losing progress? (Session)
5. Are tasks defined by what to achieve, not how to achieve it? (Outcomes)
6. Can I do everything from the command line? (Interfaces)
7. Are intermediate results saved to files I can inspect? (State)
8. Is there a way to measure if output quality improved? (Benchmarks)
9. Are there resource limits configured? (Cost)
