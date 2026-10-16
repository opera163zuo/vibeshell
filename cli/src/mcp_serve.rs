use anyhow::{bail, Context, Result};
use axum::Router;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use vibeshell_core::{mcp::McpServer, Database, SessionManager};

pub fn validate_listener(addr: SocketAddr, paired_agents: usize) -> Result<()> {
    if !addr.ip().is_loopback() && paired_agents == 0 {
        bail!("Refusing non-loopback MCP listener until at least one Agent has been paired");
    }
    Ok(())
}

fn pairing_registry(
    database: Arc<Database>,
) -> Arc<Mutex<vibeshell_core::mcp::pairing::PairingRegistry>> {
    Arc::new(Mutex::new(
        vibeshell_core::mcp::pairing::PairingRegistry::with_database(database),
    ))
}

pub fn build_router(
    database: Arc<Database>,
    session_manager: Arc<SessionManager>,
    bind: SocketAddr,
) -> Result<Router> {
    let registry = pairing_registry(database.clone());
    validate_listener(
        bind,
        registry
            .lock()
            .map_err(|_| anyhow::anyhow!("pairing lock poisoned"))?
            .list()
            .len(),
    )?;
    let mcp = McpServer::new(database, session_manager).with_pairing_registry(registry.clone());
    Ok(mcp.router_with_pairing())
}

pub fn run() -> Result<()> {
    let bind: SocketAddr = std::env::var("VIBESHELL_MCP_BIND")
        .unwrap_or_else(|_| "127.0.0.1:8765".to_string())
        .parse()
        .context("VIBESHELL_MCP_BIND must be a socket address")?;
    let database = Arc::new(Database::new().context("Failed to open VibeShell database")?);
    let pairing_registry = pairing_registry(database.clone());
    validate_listener(
        bind,
        pairing_registry
            .lock()
            .map_err(|_| anyhow::anyhow!("pairing registry lock poisoned"))?
            .list()
            .len(),
    )?;
    let mut pairing = pairing_registry
        .lock()
        .map_err(|_| anyhow::anyhow!("pairing registry lock poisoned"))?;
    let pairing_code = pairing.create_code(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    );
    drop(pairing);
    eprintln!("VibeShell MCP pairing code (5 min): {pairing_code}");
    let session_manager = Arc::new(SessionManager::new(database.clone()));
    session_manager.set_tunnel_manager(Arc::new(vibeshell_core::tunnel::TunnelManager::new()));
    session_manager.set_session_logger(Arc::new(vibeshell_core::logging::SessionLogger::new(
        database.clone(),
    )));
    let app = build_router(database, session_manager, bind)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind(bind).await?;
        axum::serve(listener, app).await?;
        Result::<()>::Ok(())
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn rejects_lan_listener_when_no_agent_is_paired() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 8765);
        assert!(validate_listener(addr, 0).is_err());
    }

    #[test]
    fn permits_loopback_listener_without_pairing() {
        let addr = "127.0.0.1:8765".parse().unwrap();
        assert!(validate_listener(addr, 0).is_ok());
    }
}
