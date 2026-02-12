use std::collections::HashMap;
use std::fs::File;
use std::io::{read_to_string, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use clap::{Parser, Subcommand};
use clap::ValueHint::{FilePath};
use clap_logflag::{LogDestinationConfig, LoggingConfig};
use log::LevelFilter;
use serde::{Deserialize, Serialize};

/// A basic cli tool to manage (configuration) files based on profiles.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Commands,
	/// profiles file
	#[arg(long, value_hint = FilePath, default_value = "profiles.toml")]
	pub config: PathBuf,
	#[clap(flatten)]
	log: clap_logflag::LogArgs,
}

#[derive(Subcommand)]
pub enum Commands {
	/// Add a file to a profile (create profile if it doesn't exist)
	Add { profile:String, file: PathBuf },
	/// Remove a file from a profile (delete profile if it's empty)
	Remove { profile:String, file: PathBuf },
	/// Activate a specific profile (de-activates all others)
	Activate { profile:String },
	/// De-activate all profiles resetting all managed files into their original state
	DeActivate,
}

#[derive(Deserialize,Serialize,Debug,Default)]
struct Profile{files:Vec<PathBuf>}

fn get_profiles(profiles:&PathBuf) -> Result<HashMap<String,Profile>,String>
{
	if !profiles.exists() {
		log::warn!(r#"Profiles file "{}" doesn't exist. Creating an empty one."#,profiles.display());
		File::create(&profiles)
			.map_err(|e|format!(r#"Profiles file "{}" could not be created ({e}). Aborting."#,profiles.display()))?;
	} else if profiles.is_dir() {
		Err(format!(r#"Profiles file "{}" is a directory. Aborting."#,profiles.display()))?;
	}

	File::open(&profiles).and_then(read_to_string)
		.map_err(|e|format!(r#"Failed opening profiles file "{}":{e}. Aborting."#,profiles.display()))
		.and_then(|s|
			toml::from_str(s.as_str()).map_err(|e|format!(r#"Failed parse profiles file "{}":{e}. Aborting."#,profiles.display()))
		)
}

fn make_canon_names(basename:&Path,profile_name:&str) -> Result<(PathBuf,PathBuf, PathBuf),String>{
	let basename= basename.canonicalize()
		.map_err(|e|format!(r#"Failed to canonicalize "{}":{e}"#, basename.display()))?;
	let mut new_filename = basename.to_owned();
	new_filename.add_extension(profile_name);
	let mut org_filename = basename.to_owned();
	org_filename.add_extension("org");
	Ok((basename,new_filename,org_filename))
}

fn copy_file(from:&Path,to:&Path) -> Result<u64, String> {
	log::debug!(r#"Creating "{}" as a copy of "{}""#,to.display(),from.display());
	std::fs::copy(from, to)
		.map_err(|e|format!(r#"Error copying "{}" to "{}": {e}"#, from.display(), to.display()))
}
fn add_profile(name:&String, basename:&Path, profiles: &mut HashMap<String,Profile>) -> Result<(),String>
{
	if name == "org" {
		return Err(r#"The profile name "org" is reserved, please use another"#.to_string())
	}

	let (basename,new_name, org_name) = make_canon_names(basename, name)?;
	copy_file(&basename, &org_name)?;
	copy_file(&basename, &new_name)?;

	profiles.entry(name.clone()).or_insert(Default::default()).files.push(basename.clone());
	log::info!(r#"Added "{}" to profile "{name}""#,basename.display());
	Ok(())
}

fn remove_profile(name:&String, basename:&Path, profiles: &mut HashMap<String,Profile>) -> Result<(),String>
{
	let (basename,new_name,org_name) = make_canon_names(basename, name)?;
	let profile = profiles.get_mut(name).ok_or(format!(r#"Profile "{name}" doesn't exist"#))?;
	let found = profile.files.iter().position(|p|p.eq(&basename))
		.ok_or(format!(r#"File "{}" not found in Profile "{name}""#, basename.display()))?;
	profile.files.remove(found);
	std::fs::remove_file(&new_name)
		.map_err(|e|format!(r#"Failed to remove file "{}": {e}"#, new_name.display()))?;
	std::fs::remove_file(&org_name)
		.map_err(|e|format!(r#"Failed to remove file "{}": {e}"#, org_name.display()))?;
	log::info!(r#"File "{}" removed from Profile "{name}""#,basename.display());
	if profile.files.is_empty(){
		log::info!(r#"Profile "{name}" is empty now, removing it.."#);
		profiles.remove(name);
	}
	Ok(())
}

fn activate(name:&String, profiles: &HashMap<String,Profile>) -> Result<(),String>
{
	let profile = profiles.get(name).ok_or(format!(r#"Profile "{name}" doesn't exist"#))?;
	log::info!(r#"Activating profile "{name}""#);
	for file in &profile.files
	{
		let (basename,new_name,_) = make_canon_names(file, name)?;
		copy_file(&new_name,&basename)?;
	}
	Ok(())
}
fn deactivate(profiles: &HashMap<String,Profile>) -> Result<(),String>
{
	log::info!("Deactivating all profiles ...");
	let files:std::collections::HashSet<_> = profiles.iter().map(|(_,p)|p.files.iter()).flatten().collect();
	for file in files
	{
		let (basename,_,org_name) = make_canon_names(file, "org")?;
		copy_file(&org_name,&basename)?;
	}
	Ok(())
}

fn main() {
	let args = Cli::parse();
	// Initialize logging with the flags from clap
	clap_logflag::init_logging!(
        args.log.or_default(LoggingConfig::new(vec![LogDestinationConfig {
                destination: clap_logflag::LogDestination::Stderr,
                level: None,
            }],)),
        LevelFilter::Info
    );

	let mut profiles = match get_profiles(&args.config)
	{
		Ok(prf) => prf,
		Err(e) => {log::error!("{e}");exit(1);}
	};

	if let Err(e) = match args.command
	{
		Commands::Add { profile,file } =>
			deactivate(&profiles).and_then(|_|add_profile(&profile,&file,&mut profiles)),
		Commands::Remove { profile,file } =>
			deactivate(&profiles).and_then(|_|remove_profile(&profile,&file,&mut profiles)),
		Commands::Activate { profile } =>
			deactivate(&profiles).and_then(|_|activate(&profile,&profiles)),
		Commands::DeActivate => deactivate(&profiles)
	}{
		log::error!("{e}");
		exit(1);
	}

	let new_cfg= match toml::to_string_pretty(&profiles)
	{
		Ok(s) => s,
		Err(e) => {
			log::error!("Failed to serialize configuration: {e}");exit(1);
		}
	};
	if let Err(e)=File::create(&args.config).and_then(|mut f|f.write_all(new_cfg.as_bytes()))
	{
		log::error!(r#"Failed writing "{}": {e}"#,args.config.display());
	}
}
