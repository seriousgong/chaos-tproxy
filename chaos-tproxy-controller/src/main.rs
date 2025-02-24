use std::process::exit;

use anyhow::anyhow;
use chaos_tproxy_proxy::proxy_main;
use chaos_tproxy_proxy::signal::Signals;
use tokio::signal::unix::SignalKind;

use crate::cmd::command_line::{get_config_from_opt, Opt};
use crate::cmd::interactive::handler::ConfigServer;
use crate::proxy::exec::Proxy;

pub mod cmd;
pub mod proxy;
pub mod raw_config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opt = match Opt::from_args_checked() {
        Err(e) => {
            println!("{}", e);
            exit(1)
        }
        Ok(o) => o,
    };
    tracing_subscriber::fmt()
        .with_max_level(opt.get_level_filter())
        .with_writer(std::io::stderr)
        .try_init()
        .map_err(|err| anyhow!("{}", err))?;

    if opt.proxy {
        proxy_main(opt.ipc_path.clone().unwrap()).await?;
    }

    if opt.input.is_some() {
        let cfg = get_config_from_opt(&opt).await?;
        let mut proxy = Proxy::new(opt.verbose);
        proxy.reload(cfg.proxy_config).await?;
        let mut signals = Signals::from_kinds(&[SignalKind::interrupt(), SignalKind::terminate()])?;
        signals.wait().await?;
        proxy.stop().await?;
        return Ok(());
    }

    if opt.interactive {
        let mut config_server = ConfigServer::new(Proxy::new(opt.verbose));
        config_server.serve_interactive();

        let mut signals = Signals::from_kinds(&[SignalKind::interrupt(), SignalKind::terminate()])?;
        signals.wait().await?;
        config_server.stop().await?;
    }
    Ok(())
}
