use std::collections::HashMap;
use std::process::Command;

use anyhow::{bail, Context, Result};

const HELPER: &str = "/usr/local/bin/tunl-helper";

#[derive(Debug, Clone, Default)]
pub struct Iface {
    pub public_key: String,
    pub listen_port: String,
    pub peers: Vec<Peer>,
}

#[derive(Debug, Clone, Default)]
pub struct Peer {
    pub public_key: String,
    pub endpoint: String,
    pub allowed_ips: String,
    pub latest_handshake: u64,
    pub rx: u64,
    pub tx: u64,
}

/// Runs the privileged helper, directly when root, otherwise via non-interactive sudo.
fn run(args: &[&str]) -> Result<String> {
    let is_root = unsafe { libc::geteuid() } == 0;
    let output = if is_root {
        Command::new(HELPER).args(args).output()
    } else {
        Command::new("sudo").arg("-n").arg(HELPER).args(args).output()
    }
    .context("failed to run tunl-helper")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = stderr.trim().lines().last().unwrap_or("unknown error");
        bail!("{}", msg);
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn list() -> Result<Vec<String>> {
    Ok(run(&["list"])?
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect())
}

pub fn status() -> Result<HashMap<String, Iface>> {
    Ok(parse_status(&run(&["status"])?))
}

pub fn up(name: &str) -> Result<()> {
    run(&["up", name]).map(|_| ())
}

pub fn down(name: &str) -> Result<()> {
    run(&["down", name]).map(|_| ())
}

/// Parses the helper's filtered `wg show all dump` output.
fn parse_status(dump: &str) -> HashMap<String, Iface> {
    let mut ifaces: HashMap<String, Iface> = HashMap::new();
    for line in dump.lines() {
        let f: Vec<&str> = line.split('\t').collect();
        match f.len() {
            4 => {
                let iface = ifaces.entry(f[0].to_owned()).or_default();
                iface.public_key = f[1].to_owned();
                iface.listen_port = f[2].to_owned();
            }
            9 => {
                ifaces.entry(f[0].to_owned()).or_default().peers.push(Peer {
                    public_key: f[1].to_owned(),
                    endpoint: f[3].to_owned(),
                    allowed_ips: f[4].to_owned(),
                    latest_handshake: f[5].parse().unwrap_or(0),
                    rx: f[6].parse().unwrap_or(0),
                    tx: f[7].parse().unwrap_or(0),
                });
            }
            _ => {}
        }
    }
    ifaces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dump() {
        let dump = "wg0\tPUB\t51820\toff\n\
                    wg0\tPEER\t-\t1.2.3.4:51820\t10.0.0.0/24\t1700000000\t100\t200\t25\n";
        let s = parse_status(dump);
        let wg0 = &s["wg0"];
        assert_eq!(wg0.public_key, "PUB");
        assert_eq!(wg0.listen_port, "51820");
        assert_eq!(wg0.peers.len(), 1);
        assert_eq!(wg0.peers[0].endpoint, "1.2.3.4:51820");
        assert_eq!(wg0.peers[0].latest_handshake, 1700000000);
        assert_eq!(wg0.peers[0].rx, 100);
        assert_eq!(wg0.peers[0].tx, 200);
    }
}
