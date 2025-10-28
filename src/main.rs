use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

type Profiles = HashMap<String, HashMap<String, String>>;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    #[serde(default)]
    profiles: Profiles,
    updated_at: i64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            profiles: HashMap::new(),
            updated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }
}

struct Gakun {
    config: Config,
    config_path: PathBuf,
    ssh_config_path: PathBuf,
}

impl Gakun {
    fn new() -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
        let config_path = home.join(".config/gakun/config.json");
        let ssh_config_path = home.join(".ssh/config");

        let mut gakun = Gakun {
            config: Config::default(),
            config_path,
            ssh_config_path,
        };

        gakun.load_config()?;
        Ok(gakun)
    }

    fn load_config(&mut self) -> Result<()> {
        // Create directory if it doesn't exist
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        // Try to read existing config
        match fs::read_to_string(&self.config_path) {
            Ok(data) => {
                self.config = serde_json::from_str(&data)
                    .context("Failed to parse config file")?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // Create new config file
                self.save_config()?;
            }
            Err(e) => return Err(e.into()),
        }

        Ok(())
    }

    fn save_config(&mut self) -> Result<()> {
        self.config.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let json = serde_json::to_string(&self.config)
            .context("Failed to serialize config")?;

        fs::write(&self.config_path, json)
            .context("Failed to write config file")?;

        Ok(())
    }

    fn add(&mut self, profile: &str, host: &str, key: &str) -> Result<()> {
        // Validate that the key file exists
        fs::metadata(key)
            .with_context(|| format!("SSH key path is not valid: {}", key))?;

        // Add to config
        self.config.profiles
            .entry(profile.to_string())
            .or_insert_with(HashMap::new)
            .insert(host.to_string(), key.to_string());

        self.save_config()?;

        Ok(())
    }

    fn use_profile(&mut self, profile: &str, host: &str) -> Result<()> {
        let key = self.config.profiles
            .get(profile)
            .and_then(|hosts| hosts.get(host))
            .ok_or_else(|| anyhow!(
                "There is no such profile and host combination. Please type gakun ls to show your profiles and hosts."
            ))?;

        let data = self.read_file_with_skip_section()?;

        let new_config = format!(
            "###### gakun begin\nHost {}\n  Hostname {}\n  IdentityFile {}\n###### gakun end\n",
            host, host, key
        );

        fs::write(&self.ssh_config_path, format!("{}{}", new_config, data))
            .context("Failed to write SSH config")?;

        println!("Key {} is now active for {} ✓", key, host);

        Ok(())
    }

    fn list(&self) -> Result<()> {
        for (profile, hosts) in &self.config.profiles {
            println!("\n{}:", profile);
            for (host, key) in hosts {
                println!("   {} → {}", host, key);
            }
        }
        Ok(())
    }

    fn read_file_with_skip_section(&self) -> Result<String> {
        let file = File::open(&self.ssh_config_path)
            .or_else(|_| {
                // If file doesn't exist, create it
                File::create(&self.ssh_config_path)
            })
            .context("Failed to open SSH config file")?;

        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        let mut skip_section = false;

        for line in reader.lines() {
            let line = line?;

            if line.contains("gakun begin") {
                skip_section = true;
                continue;
            }

            if line.contains("gakun end") {
                skip_section = false;
                continue;
            }

            if !skip_section {
                lines.push(line);
            }
        }

        // Join lines with newlines, preserving structure
        Ok(lines.join("\n") + if lines.is_empty() { "" } else { "\n" })
    }

    fn detach(&self) -> Result<()> {
        // Read the current SSH config and remove gakun-managed section
        let data = self.read_file_with_skip_section()?;

        // Write back the cleaned config
        fs::write(&self.ssh_config_path, data)
            .context("Failed to write SSH config")?;

        println!("Gakun section removed from {} ✓", self.ssh_config_path.display());

        Ok(())
    }
}

#[derive(Parser)]
#[command(name = "gakun")]
#[command(about = "SSH key manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Add host and key to a profile. Example: 'gakun add work gitlab.com ~/.ssh/id_rsa_work'")]
    Add {
        /// Profile name
        profile: String,
        /// Host to configure
        #[arg(short = 'h', long)]
        host: String,
        /// Path to SSH key
        #[arg(short = 'k', long)]
        key: String,
    },
    #[command(about = "Use SSH key for certain host. Example: 'gakun use work -h gitlab.com'")]
    Use {
        /// Profile name
        profile: String,
        /// Host to configure
        #[arg(short = 'h', long)]
        host: String,
    },
    #[command(about = "List profiles")]
    Ls,
    #[command(about = "Detach gakun - remove gakun-managed section from ~/.ssh/config")]
    #[command(alias = "d")]
    Detach,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut gakun = Gakun::new()?;

    match cli.command {
        Commands::Add { profile, host, key } => {
            gakun.add(&profile, &host, &key)?;
        }
        Commands::Use { profile, host } => {
            gakun.use_profile(&profile, &host)?;
        }
        Commands::Ls => {
            gakun.list()?;
        }
        Commands::Detach => {
            gakun.detach()?;
        }
    }

    Ok(())
}
