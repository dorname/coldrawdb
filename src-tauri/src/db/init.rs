use rusqlite::Connection;
use std::path::Path;

// 初始化数据库
pub fn init_db() -> Result<Connection, rusqlite::Error> {
    let path = Path::new("db.sqlite");
    let conn = Connection::open(path)?;
    Ok(conn)
}
