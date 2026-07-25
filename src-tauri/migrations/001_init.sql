CREATE TABLE IF NOT EXISTS projects (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    path        TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    archived_at INTEGER,
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS tags (
    id    INTEGER PRIMARY KEY AUTOINCREMENT,
    name  TEXT NOT NULL UNIQUE,
    color TEXT NOT NULL DEFAULT '#3b82f6'
);

CREATE TABLE IF NOT EXISTS project_tags (
    project_id INTEGER NOT NULL,
    tag_id     INTEGER NOT NULL,
    PRIMARY KEY (project_id, tag_id),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id)     REFERENCES tags(id)     ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS custom_commands (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  INTEGER NOT NULL,
    name        TEXT NOT NULL,
    command     TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    icon        TEXT NOT NULL DEFAULT '',
    sort_order  INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    UNIQUE (project_id, name)
);

CREATE TABLE IF NOT EXISTS settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_custom_commands_project ON custom_commands(project_id);

CREATE TABLE IF NOT EXISTS report_history (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project_ids TEXT NOT NULL,
    date_from   TEXT NOT NULL,
    date_to     TEXT NOT NULL,
    range_label TEXT NOT NULL DEFAULT '',
    author_mode TEXT NOT NULL DEFAULT 'me',
    language    TEXT NOT NULL DEFAULT 'zh-CN',
    period_type TEXT NOT NULL DEFAULT 'daily',
    result      TEXT NOT NULL,
    created_at  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS report_commits (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    report_id           INTEGER NOT NULL,
    project_id          INTEGER,
    project_name        TEXT NOT NULL,
    project_description TEXT NOT NULL DEFAULT '',
    commit_data         TEXT NOT NULL,
    FOREIGN KEY (report_id) REFERENCES report_history(id) ON DELETE CASCADE,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_report_history_created ON report_history(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_report_commits_report ON report_commits(report_id);

CREATE TABLE IF NOT EXISTS report_schedules (
    id                   TEXT PRIMARY KEY,
    name                 TEXT NOT NULL DEFAULT '',
    enabled              INTEGER NOT NULL DEFAULT 1,
    report_type          TEXT NOT NULL DEFAULT 'daily',
    project_ids          TEXT NOT NULL,
    author_mode          TEXT NOT NULL DEFAULT 'me',
    time_of_day          TEXT NOT NULL,
    weekdays_only        INTEGER NOT NULL DEFAULT 0,
    chinese_workday_only INTEGER NOT NULL DEFAULT 0,
    weekly_workweek      INTEGER NOT NULL DEFAULT 1,
    weekly_start_weekday INTEGER NOT NULL DEFAULT 1,
    weekly_end_weekday   INTEGER NOT NULL DEFAULT 5,
    last_run_at          INTEGER
);

-- 项目维度被隐藏的 UI 项(npm script 文件/命令、compose 文件)
CREATE TABLE IF NOT EXISTS hidden_items (
    project_id INTEGER NOT NULL,
    kind       TEXT NOT NULL,
    target_key TEXT NOT NULL,
    PRIMARY KEY (project_id, kind, target_key),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- 代码托管平台账号绑定(GitHub / Gitee / 自建 GitLab),token 明文存储(仅本机使用)
CREATE TABLE IF NOT EXISTS git_accounts (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    provider   TEXT NOT NULL,
    label      TEXT NOT NULL DEFAULT '',
    base_url   TEXT NOT NULL,
    username   TEXT NOT NULL DEFAULT '',
    token      TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

