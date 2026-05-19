# TaskFlow
 
A production-style task management system built as a pet project to explore Rust microservices. Features three independent services communicating over a pluggable message broker, all wired together with Docker Compose.
 
---

 
### Services
 
| Service | Port | Responsibility |
|---|---|---|
| `user-service` | 3001 | Registration, login, JWT issuance |
| `task-service` | 3002 | Task CRUD, ownership, pagination |
| `notification-service` | 3003 | Consumes broker events, sends notifications |
 
 
 
## Getting Started
### Run with Docker Compose
 
```bash
git clone https://github.com/rmnvalera/Pet-todo-Rust.git
cd Pet-todo-Rust
docker compose up --build
```
 
This starts PostgreSQL, RabbitMQ, and all three services.
 
### Run locally
 
1. Start PostgreSQL and your preferred message broker.
2. Edit the config common Settings.yaml:
3. Run a service:
```bash
cargo run -p task-service
```
 
---
 
## Configuration
 
Each service is configured via a YAML file. Example:
 
```yaml
port: 3002
 
db:
  url: postgres://postgres:changeme@localhost:5432/todo
 
jwt:
  secret: secret
  deadline: 24h
 
# Switch between RabbitMQ and NATS by uncommenting the desired block:
 
# messaging:
#   provider: nats
#   url: nats://localhost:4222
 
messaging:
  provider: rabbitmq
  url: amqp://user:password@localhost:5672
```
 
Changing `provider` to `nats` and updating the URL is all that's needed to switch brokers — no code changes required.
