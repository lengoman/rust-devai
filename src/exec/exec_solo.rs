use crate::agent::{load_base_agent_config, Agent, AgentDoc};
use crate::ai::{get_genai_client, run_solo_agent};
use crate::exec::SoloConfig;
use crate::hub::get_hub;
use crate::{Error, Result};
use simple_fs::{watch, SEventKind, SFile};

/// Executes the Run command
/// Can either perform a single run or run in watch mode
pub async fn exec_solo<T>(solo_config: T) -> Result<()>
where
	T: TryInto<SoloConfig, Error = Error>,
{
	let solo_config = solo_config.try_into()?;

	// -- Get the AI client and agent
	let client = get_genai_client()?;
	let hub = get_hub();

	// -- If NOT in watch mode, then just run once
	if !solo_config.watch() {
		let agent = load_solo_agent(&solo_config)?;
		run_solo_agent(&client, &agent, (&solo_config).into()).await?;
	}
	// -- If in watch mode
	else {
		// Do the first run
		let agent = load_solo_agent(&solo_config)?;
		match run_solo_agent(&client, &agent, (&solo_config).into()).await {
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
						get_hub()
							.publish(format!(
								"\nSolo Agent Modified '{}', running again.",
								solo_config.solo_path()
							))
							.await;
						// Ensure to reload the agent
						let agent = load_solo_agent(&solo_config)?;

						match run_solo_agent(&client, &agent, (&solo_config).into()).await {
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

fn load_solo_agent(solo_config: &SoloConfig) -> Result<Agent> {
	// TODO: Create it if solo_config.create_if_needed with the eventual template

	let solo_file = SFile::new(solo_config.solo_path().path()).map_err(|err| format!("Solo file not found: {err}"))?;
	let base_config = load_base_agent_config()?;

	let agent_doc = AgentDoc::from_file(solo_file)?;
	agent_doc.into_agent(base_config)
}

// endregion: --- Support
