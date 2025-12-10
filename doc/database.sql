-- =====================================================
-- DM-Rust 数据库初始化脚本
-- 请先创建数据库，然后执行此脚本
-- =====================================================

-- 创建数据库（如果不存在）
-- CREATE DATABASE IF NOT EXISTS dm_rust DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
-- USE dm_rust;

-- =====================================================
-- lspc_screen 表 - 屏幕配置
-- =====================================================
CREATE TABLE IF NOT EXISTS `lspc_screen` (
    `id` VARCHAR(64) NOT NULL PRIMARY KEY COMMENT '主键ID（UUID）',
    `type` VARCHAR(32) NOT NULL COMMENT '类型：Clean, Close, Normal, Pause, Register, Vote',
    `name` VARCHAR(255) NOT NULL COMMENT '名称',
    `content` TEXT NOT NULL COMMENT '内容（JSON或文本）',
    `active` TINYINT(1) NOT NULL DEFAULT 0 COMMENT '是否激活',
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    INDEX `idx_type` (`type`),
    INDEX `idx_active` (`active`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='屏幕配置表';

-- =====================================================
-- lspc_material 表 - 素材管理（合并了素材和资源信息）
-- =====================================================
CREATE TABLE IF NOT EXISTS `lspc_material` (
    `id` VARCHAR(64) NOT NULL PRIMARY KEY COMMENT '主键ID（UUID）',
    `name` VARCHAR(255) NOT NULL COMMENT '素材名称',
    `screen_id` VARCHAR(64) NOT NULL DEFAULT '' COMMENT '关联的屏幕ID',
    `preset` TINYINT(1) NOT NULL DEFAULT 0 COMMENT '是否为预设素材',
    `path` VARCHAR(1024) NOT NULL DEFAULT '' COMMENT '文件路径（相对于静态目录）',
    `resource_type` VARCHAR(32) NOT NULL DEFAULT '' COMMENT '资源类型：image, video, audio, document, other',
    `size` BIGINT NOT NULL DEFAULT 0 COMMENT '文件大小（字节）',
    `mime_type` VARCHAR(128) NOT NULL DEFAULT '' COMMENT 'MIME类型',
    `original_name` VARCHAR(255) NOT NULL DEFAULT '' COMMENT '原始文件名',
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    INDEX `idx_name` (`name`),
    INDEX `idx_screen_id` (`screen_id`),
    INDEX `idx_preset` (`preset`),
    INDEX `idx_resource_type` (`resource_type`),
    INDEX `idx_path` (`path`(255))
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='素材管理表';

-- =====================================================
-- 示例数据（可选）
-- =====================================================

-- 插入示例屏幕配置
-- INSERT INTO `lspc_screen` (`id`, `type`, `name`, `content`, `active`) VALUES
-- (UUID(), 'Normal', '默认屏幕', '{"title": "欢迎使用", "background": "#ffffff"}', 1),
-- (UUID(), 'Vote', '投票屏幕', '{"options": ["选项A", "选项B", "选项C"]}', 0);

-- 插入示例素材
-- INSERT INTO `lspc_material` (`id`, `name`, `screen_id`, `path`, `resource_type`) VALUES
-- (UUID(), '示例图片', '', '202501/example.png', 'image'),
-- (UUID(), '示例视频', '', '202501/example.mp4', 'video');
