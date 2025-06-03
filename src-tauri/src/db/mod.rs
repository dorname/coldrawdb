mod init;
pub mod models;
pub use init::init_db;
use rusqlite::{Connection, Params};
use std::sync::{Arc, Mutex, MutexGuard};
pub struct DB {
    conn: Arc<Mutex<Connection>>,
}

impl DB {
    pub fn init() -> Result<Self, rusqlite::Error> {
        let conn = init_db()?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
    /// 获取数据库连接的互斥锁 Guard
    pub async fn get_conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().expect("Failed to lock DB connection")
    }

    /// 获取数据库连接的互斥锁 Guard
    pub fn get_conn_sync(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().expect("Failed to lock DB connection")
    }

    /// 执行 SQL 语句
    pub async fn execute<P: Params>(&self, sql: &str,params:P) -> Result<(), rusqlite::Error> {
        let conn = self.get_conn().await;
        conn.execute(sql, params)?;
        Ok(())
    }

    /// 执行 SQL 语句
    pub fn execute_sync<P: Params>(&self, sql: &str,params:P) -> Result<(), rusqlite::Error> {
        let conn = self.get_conn_sync();
        conn.execute(sql, params)?;
        Ok(())
    }

    /// 在数据库连接上执行操作
    /// 闭包函数，接受一个数据库连接的引用，并返回一个结果
    pub async fn with_connection<F, T>(&self, f: F) -> Result<T, rusqlite::Error>
    where
        F: FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    {
        let conn = self.get_conn().await;
        f(&conn)
    }

}

// 测试模块
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_db() {
        let db = DB::init().unwrap();

        // 先删除表
        db.execute("DROP TABLE IF EXISTS users",[]).await.unwrap();

        // 创建表
        db.execute("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, email TEXT NOT NULL UNIQUE)",[])
            .await
            .unwrap();

        // 插入数据
        db.execute("INSERT INTO users (name, email) VALUES ('test1', 'test1@example.com')",[])
            .await
            .unwrap();

        // 查询数据
        let result = db
            .with_connection(|conn| {
                let mut stmt = conn.prepare("SELECT * FROM users where name = ?")?;
                let users = stmt.query_map(["test1"], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })?;
                Ok(users.collect::<Result<Vec<_>, _>>()?)
            })
            .await
            .unwrap();
        println!("result: {:?}", result);
        assert!(!result.is_empty());
    }

    // 测试 with_connection 方法
    #[tokio::test]
    async fn test_with_connection() {
        let db = DB::init().unwrap();
          // 插入数据
          db.execute("INSERT INTO users (name, email) VALUES (?, ?)",["test2","test2@example.com"])
          .await
          .unwrap();
        // 查询数据
        let result = db
            .with_connection(|conn| {
                let mut stmt = conn.prepare("SELECT * FROM users where name = ?")?;
                let users = stmt.query_map(["test2"], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })?;
                Ok(users.collect::<Result<Vec<_>, _>>()?)
            })
            .await
            .unwrap();
        println!("result: {:?}", result);
    }
}
