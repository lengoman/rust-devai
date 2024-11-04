use crate::agent::{get_solo_and_target_path, load_base_agent_config, Agent, AgentDoc};
use crate::ai::{get_genai_client, run_solo_agent};
use crate::cli::SoloArgs;
use crate::exec::support::open_vscode;
use crate::hub::get_hub;
use crate::support::{DirContext, RunSoloOptions};
use crate::Result;
use simple_fs::{watch, SEventKind, SFile};
use std::path::Path;

/// Executes the Run command
/// Can either perform a single run or run in watch mode
pub async fn exec_solo(solo_args: SoloArgs, dir_context: DirContext) -> Result<()> {
	// -- Get the AI client and agent
	let client = get_genai_client()?;
	let hub = get_hub();

	let (solo_path, target_path) = get_solo_and_target_path(&solo_args.path)?;
	let agent = load_solo_agent(solo_path.path(), &dir_context)?;
	let solo_options = RunSoloOptions::new(solo_args, &dir_context, &agent, target_path)?;

	if solo_options.base_run_config().open() {
		open_vscode(agent.file_path()).await;
		open_vscode(solo_options.target_path()).await;
	}

	// -- If NOT in watch mode, then just run once
	if !solo_options.base_run_config().watch() {
		run_solo_agent(&client, &agent, &solo_options).await?;
	}
	// -- If in watch mode
	else {
		// Do the first run
		let agent = load_solo_agent(agent.file_path(), &dir_context)?;
		match run_solo_agent(&client, &agent, &solo_options).await {
			Ok(_) => (),
			Err(err) => hub.publish(format!("ERROR: {}", err)).await,
		}

		// And watch for modifications
		let watcher = watch(agent.file_path())?;
		loop {
			match watcher.rx.recv() {
				Ok(events) => {
					// If there is a modification, then run again
					let has_modify = events.iter().any(|evt| matches!(evt.skind, SEventKind::Modify));
					if has_modify {
						hub.publish(format!("\nSolo Agent Modified '{}', running again.", agent.file_path()))
							.await;
						// Ensure to reload the agent
						let agent = load_solo_agent(agent.file_path(), &dir_context)?;

						match run_solo_agent(&client, &agent, &solo_options).await {
							Ok(_) => (),
							Err(err) => hub.publish(format!("ERROR: {}", err)).await,
						}
					}
				}
				Err(err) => {
					// Handle any errors related to receiving the message
					hub.publish(format!("Error receiving event: {:?}", err)).await;
					break;
				}
			}
		}
	}

	Ok(())
}

// region:    --- Support

fn load_solo_agent(solo_agent_path: impl AsRef<Path>, dir_context: &DirContext) -> Result<Agent> {
	let solo_agent_path = solo_agent_path.as_ref();

	let solo_file = SFile::new(solo_agent_path).map_err(|err| format!("Solo file not found: {err}"))?;
	let base_config = load_base_agent_config(dir_context)?;

	let agent_doc = AgentDoc::from_file(solo_file)?;
	agent_doc.into_agent(base_config)
}

// endregion: --- Support
