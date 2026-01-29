-- 添加 fixed 字段到 lspc_screen 表
-- fixed: 是否固定（0: 否, 1: 是），默认值为 0
ALTER TABLE lspc_screen ADD COLUMN fixed TINYINT NOT NULL DEFAULT 0 COMMENT '是否固定（0: 否, 1: 是）';
