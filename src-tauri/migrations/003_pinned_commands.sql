-- App version: 0.1.5
-- Status: in development

-- 常用命令标记:kind 区分来源(npm script / compose 文件 / compose 服务 / 自定义命令),
-- target_key 在同一项目同一 kind 下唯一;label/command 为标记时快照,托盘弹窗直接执行
CREATE TABLE IF NOT EXISTS pinned_commands (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id  INTEGER NOT NULL,
    kind        TEXT NOT NULL,
    target_key  TEXT NOT NULL,
    label       TEXT NOT NULL,
    command     TEXT NOT NULL,
    cwd         TEXT,
    created_at  INTEGER NOT NULL,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    UNIQUE (project_id, kind, target_key)
);
CREATE INDEX IF NOT EXISTS idx_pinned_commands_project ON pinned_commands(project_id);
