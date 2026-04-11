-- Add migration script here
CREATE TYPE task_service.task_status AS ENUM ('todo', 'in_progress', 'done');
CREATE TYPE task_service.task_priority AS ENUM ('low', 'medium', 'high');

CREATE TABLE task_service.tasks
(
    id          UUID PRIMARY KEY                    DEFAULT gen_random_uuid(),
    title       VARCHAR(255)               NOT NULL,
    description TEXT,
    status      task_service.task_status   NOT NULL DEFAULT 'todo',
    priority    task_service.task_priority NOT NULL DEFAULT 'medium',
    owner_id    UUID                       NOT NULL,
    created_at  TIMESTAMPTZ                         DEFAULT NOW(),
    updated_at  TIMESTAMPTZ                         DEFAULT NOW()
);