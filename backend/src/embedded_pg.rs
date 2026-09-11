//! feat-db-connect-import-and-ddl-realdb-verify：嵌入式 PG 测试支撑（仅 #[cfg(test)]）。
//!
//! 为什么不用 postgresql_embedded crate：本环境以 root 运行，PG `initdb`/`postgres`
//! 拒绝 root；且 0.20 系要求 rustc 1.94（本机 1.89）。这里直连 Maven Central 拉
//! zonky 二进制（首次下载后缓存于 ~/.cache/cdb-pg/），子进程用 `setpriv` 降权到
//! nobody 运行，测试结束 fast-shutdown。

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

const PG_VERSION: &str = "17.5.0";
const NOBODY: &str = "65534";

pub struct EmbeddedPg {
    child: Child,
    pub url: String,
    data_dir: PathBuf,
}

impl EmbeddedPg {
    /// 启动嵌入式 PG，返回连接 URL（trust 认证，postgres 库）
    pub fn start() -> Result<Self, String> {
        let install = Self::ensure_binaries()?;
        let data_dir = std::env::temp_dir().join(format!("cdb-pg-data-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
        // initdb/postgres 以 nobody 运行：目录所有权移交
        run_ok("chown", &["-R", &format!("{NOBODY}:{NOBODY}"), &data_dir.to_string_lossy()])?;

        run_as_nobody(
            &install.join("bin/initdb"),
            &[
                "-D", &data_dir.to_string_lossy(),
                "-U", "postgres",
                "--auth=trust",
                "-E", "UTF8",
            ],
            &install,
        )?;

        let port = free_port()?;
        let child = Command::new("setpriv")
            .args([
                &format!("--reuid={NOBODY}"),
                &format!("--regid={NOBODY}"),
                "--clear-groups",
            ])
            // zonky 二进制的 libpq/libcrypto 等动态库在 <install>/lib，须显式 LD_LIBRARY_PATH
            .env("LD_LIBRARY_PATH", install.join("lib"))
            .arg(install.join("bin/postgres"))
            .args([
                "-D", &data_dir.to_string_lossy(),
                "-p", &port.to_string(),
                "-k", &data_dir.to_string_lossy(), // unix socket 目录（nobody 可写）
                "-c", "listen_addresses=127.0.0.1",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("spawn postgres: {e}"))?;

        let url = format!("postgres://postgres@127.0.0.1:{port}/postgres");
        Ok(Self {
            child,
            url,
            data_dir,
        })
    }

    pub async fn wait_ready(&self) -> Result<(), String> {
        for _ in 0..60 {
            if sqlx::PgPool::connect(&self.url).await.is_ok() {
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        Err("embedded pg 30s 内未就绪".into())
    }

    /// 下载并解压 zonky PG 二进制（jar 内嵌 .txz），缓存复用
    fn ensure_binaries() -> Result<PathBuf, String> {
        let base = dirs_home().join(format!(".cache/cdb-pg/{PG_VERSION}"));
        let marker = base.join("bin/postgres");
        if marker.exists() {
            return Ok(base);
        }
        std::fs::create_dir_all(&base).map_err(|e| e.to_string())?;
        let jar = base.join("pg.jar");
        let url = format!(
            "https://repo1.maven.org/maven2/io/zonky/test/postgres/embedded-postgres-binaries-linux-amd64/{PG_VERSION}/embedded-postgres-binaries-linux-amd64-{PG_VERSION}.jar"
        );
        run_ok("curl", &["-fsSL", &url, "-o", &jar.to_string_lossy()])?;
        run_ok("unzip", &["-o", "-q", &jar.to_string_lossy(), "-d", &base.to_string_lossy()])?;
        // jar 内为 postgres-linux-x86_64.txz
        let txz = std::fs::read_dir(&base)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .find(|p| p.extension().map(|e| e == "txz").unwrap_or(false))
            .ok_or("zonky jar 内未找到 .txz")?;
        run_ok("tar", &["-xJf", &txz.to_string_lossy(), "-C", &base.to_string_lossy()])?;
        // txz 解压出 pgsql/ 子目录：摊平到 base
        let inner = base.join("pgsql");
        if inner.exists() {
            for entry in std::fs::read_dir(&inner).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let target = base.join(entry.file_name());
                if !target.exists() {
                    std::fs::rename(entry.path(), target).map_err(|e| e.to_string())?;
                }
            }
        }
        if !marker.exists() {
            return Err(format!("PG 二进制解压后缺 bin/postgres：{}", base.display()));
        }
        Ok(base)
    }
}

impl Drop for EmbeddedPg {
    fn drop(&mut self) {
        // SIGINT = fast shutdown
        let _ = unsafe { libc_kill(self.child.id() as i32, 2) };
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.data_dir);
    }
}

/// 不引 libc crate：用 kill 命令发 SIGINT
fn libc_kill(pid: i32, _sig: i32) -> Result<(), String> {
    run_ok("kill", &["-INT", &pid.to_string()])
}

fn free_port() -> Result<u16, String> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    Ok(listener.local_addr().map_err(|e| e.to_string())?.port())
}

fn dirs_home() -> PathBuf {
    // 必须放在 nobody 可遍历的位置（/root 为 700，降权后 EACCES）；/tmp 全局可写
    std::env::temp_dir()
}

fn run_ok(cmd: &str, args: &[&str]) -> Result<(), String> {
    let out = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("{cmd} spawn: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{cmd} {:?} 失败：{}",
            args,
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

fn run_as_nobody(bin: &Path, args: &[&str], install: &Path) -> Result<(), String> {
    let out = Command::new("setpriv")
        .args([
            &format!("--reuid={NOBODY}"),
            &format!("--regid={NOBODY}"),
            "--clear-groups",
        ])
        .env("LD_LIBRARY_PATH", install.join("lib"))
        .arg(bin)
        .args(args)
        .output()
        .map_err(|e| format!("initdb spawn: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "initdb 失败：{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}
