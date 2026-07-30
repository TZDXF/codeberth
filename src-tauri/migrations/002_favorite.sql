-- App version: 0.1.5
-- Status: in development

-- 收藏标记:NULL = 未收藏,非 NULL = 收藏时间(列表中收藏项目置顶,组内按收藏时间倒序)
ALTER TABLE projects ADD COLUMN favorited_at INTEGER;
