use derive_more::derive::Display;
use derive_more::From;
use tokio::runtime::TryCurrentError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Display)]
pub enum Error {
	// -- Cli Command
	#[display("Command Agent not found at: {_0}")]
	CommandAgentNotFound(String),

	// -- Agent
	ModelMissing {
		agent_path: String,
	},

	// -- Run
	BeforeAllFailWrongReturn {
		cause: String,
	},

	// -- TokioSync
	TokioTryCurrent(TryCurrentError),

	// -- Externals
	Lua(String),
	#[from]
	Toml(toml::de::Error),
	#[from]
	JsonValueExt(value_ext::JsonValueExtError),
	#[from]
	Serde(serde_json::Error),
	#[from]
	Handlebars(handlebars::RenderError),
	#[from]
	GenAI(genai::Error),
	#[from]
	SimpleFs(simple_fs::Error),
	#[from]
	Keyring(keyring::Error),
	#[from]
	Clap(clap::error::Error),
	#[from]
	Reqwest(reqwest::Error),
	#[from]
	Io(std::io::Error),

	// -- Custom
	#[display("Error::Custom\n{_0}")]
	#[from]
	Custom(String),

	#[display("Error: {_0}\n\tCause: {_1}")]
	CustomAndCause(String, String),
}

// region:    --- Custom

impl Error {
	pub fn custom(val: impl std::fmt::Display) -> Self {
		Self::Custom(val.to_string())
	}

	pub fn custom_and_cause(context: impl Into<String>, cause: impl std::fmt::Display) -> Self {
		Self::CustomAndCause(context.into(), cause.to_string())
	}

	/// Same as custom_and_cause (just a "cute" shorcut)
	pub fn cc(context: impl Into<String>, cause: impl std::fmt::Display) -> Self {
		Self::CustomAndCause(context.into(), cause.to_string())
	}
}

impl From<&str> for Error {
	fn from(val: &str) -> Self {
		Self::Custom(val.to_string())
	}
}

// endregion: --- Custom

// region:    --- Error Boilerplate

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
