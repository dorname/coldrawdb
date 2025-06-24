/* 代办事项表 */
CREATE TABLE IF NOT EXISTS "task" (
	-- 主键
	"id" VARCHAR NOT NULL UNIQUE,
	-- 完成标记
	"complete" BOOLEAN,
	-- 排序号
	"order" INTEGER,
	-- 详情
	"details" VARCHAR,
	-- 标题
	"title" VARCHAR,
	PRIMARY KEY("id")
);

/* 图表数据结构 */
CREATE TABLE IF NOT EXISTS "diagram" (
	"id" VARCHAR NOT NULL UNIQUE,
	-- 空间缩放比例
	"zoom" VARCHAR,
	-- 设计名称
	"name" VARCHAR,
	-- 数据库名称
	"database" VARCHAR,
	-- 最近修改日期
	"lastModified" TIME,
	PRIMARY KEY("id")
);

/* 关联关系表 */
CREATE TABLE IF NOT EXISTS "diagram_link" (
	"id" VARCHAR NOT NULL UNIQUE,
	-- 图id
	"diagram_id" VARCHAR,
	"task_id" VARCHAR,
	"table_id" VARCHAR,
	"area_id" VARCHAR,
	"note_id" VARCHAR,
	"reference" VARCHAR,
	PRIMARY KEY("id")
);

CREATE INDEX IF NOT EXISTS "diagram_link_index_0"
ON "diagram_link" ("diagram_id", "task_id", "table_id");
/* 表的存储结构 */
CREATE TABLE IF NOT EXISTS "table" (
	"id" VARCHAR NOT NULL UNIQUE,
	-- 表的颜色样式
	"color" VARCHAR,
	-- 备注
	"comment" VARCHAR,
	-- 在无限画布上是否锁定
	"locked" BOOLEAN,
	-- 表名称
	"name" VARCHAR,
	-- 无限画布上横向的位置
	"x" NUMERIC,
	-- 无限画布上的纵向位置
	"y" NUMERIC,
	PRIMARY KEY("id")
);

/* 字段报表 */
CREATE TABLE IF NOT EXISTS "field" (
	"id" VARCHAR NOT NULL UNIQUE,
	-- 检查表达式字段
	"check" VARCHAR,
	-- 注释字段
	"comment" VARCHAR,
	-- 默认值字段
	"default" VARCHAR,
	-- 自增标记字段
	"increment" BOOLEAN,
	-- 非空标记
	"not_null" BOOLEAN,
	"primary" BOOLEAN,
	-- 字段大小
	"size" VARCHAR,
	-- 类型字段
	"type" VARCHAR,
	-- 索引标记
	"unique" BOOLEAN,
	"name" VARCHAR,
	PRIMARY KEY("id")
);

CREATE TABLE IF NOT EXISTS "table_link" (
	"id" VARCHAR NOT NULL UNIQUE,
	"field_id" VARCHAR,
	"table_id" VARCHAR,
	PRIMARY KEY("id")
);

/* 索引表 */
CREATE TABLE IF NOT EXISTS "indice" (
	-- 主键
	"id" VARCHAR NOT NULL UNIQUE,
	-- 索引名称
	"name" VARCHAR,
	-- 唯一索引标记
	"unique" BOOLEAN,
	PRIMARY KEY("id")
);

CREATE TABLE IF NOT EXISTS "indice_link" (
	"id" VARCHAR NOT NULL UNIQUE,
	"field_id" VARCHAR,
	"indice_id" VARCHAR,
	PRIMARY KEY("id")
);

CREATE TABLE IF NOT EXISTS "reference" (
	"id" VARCHAR NOT NULL UNIQUE,
	-- 关系映射
	"cardinality" VARCHAR,
	"deleteConstraint" VARCHAR,
	"endFieldId" VARCHAR,
	"endTableId" VARCHAR,
	"name" VARCHAR,
	"startFieldId" VARCHAR,
	"startTableId" VARCHAR,
	"updateConstraint" VARCHAR,
	PRIMARY KEY("id")
);

/* 主题区域表结构 */
CREATE TABLE IF NOT EXISTS "area" (
	"id" VARCHAR NOT NULL UNIQUE,
	-- 颜色
	"color" VARCHAR,
	-- 主题域高度
	"height" NUMERIC,
	-- 主题域名称
	"name" VARCHAR,
	-- 主题域宽度
	"width" NUMERIC,
	-- 主题域的横坐标
	"x" NUMERIC,
	-- 主题域的纵坐标
	"y" NUMERIC,
	PRIMARY KEY("id")
);

/* 注释表结构 */
CREATE TABLE IF NOT EXISTS "note" (
	"id" VARCHAR NOT NULL UNIQUE,
	-- 颜色字段
	"color" VARCHAR,
	-- 内容字段
	"content" VARCHAR,
	-- 高度字段
	"height" NUMERIC,
	-- 注释标题
	"title" VARCHAR,
	-- 注释在画布的横坐标
	"x" NUMERIC,
	-- 注释在画布的纵坐标
	"y" NUMERIC,
	PRIMARY KEY("id") 
);
