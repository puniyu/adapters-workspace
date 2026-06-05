mod common;
mod input;

use std::sync::{Arc, OnceLock};

use log::info;
use puniyu_adapter::{
	AdapterApi, AdapterCommunication, AdapterInfo, AdapterPlatform, AdapterProtocol,
	AdapterStandard, SendMsgType, adapter_info, app_name, pkg_name, pkg_version, prelude::*,
};

use crate::common::make_random_id;

pub(crate) const VERSION: puniyu_adapter::Version = pkg_version!();
pub(crate) const NAME: &str = pkg_name!();

struct OutputMessage {
	contact: String,
	message: String,
}

static OUTPUT_TX: OnceLock<tokio::sync::mpsc::UnboundedSender<OutputMessage>> = OnceLock::new();

#[adapter]
struct ConsoleAdapter;

#[adapter]
impl ConsoleAdapter {
	#[on_load]
	async fn on_load() -> puniyu_adapter::result::Result {
		let adapter = Arc::new(Adapter);
		let info = adapter.adapter_info();
		let adapter_runtime = AdapterRuntime::new(adapter);
		let bot_runtime = BotRuntime::new(adapter_runtime);
		if let Ok(bot_id) = register_bot!(runtime: bot_runtime) {
			info!("{} v{} 初始化完成", info.name, info.version);
			let bot = BotRegistry::get_with_index(bot_id).unwrap();
			let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
			std::thread::spawn(move || {
				use std::io::BufRead;
				let stdin = std::io::stdin();
				for line in stdin.lock().lines() {
					match line {
						Ok(s) => {
							let _ = tx.send(s);
						}
						Err(_) => break,
					}
				}
			});
			tokio::spawn(async move {
				while let Some(message) = rx.recv().await {
					if matches!(message.as_str(), "quit" | "exit" | "q") {
						break;
					}

					let parsed = input::parse_console_input(&message);
					common::dispatch_event(bot.as_ref(), &parsed).await;
				}
			});

			let (out_tx, mut out_rx) = tokio::sync::mpsc::unbounded_channel::<OutputMessage>();
			let _ = OUTPUT_TX.set(out_tx);
			tokio::spawn(async move {
				while let Some(message) = out_rx.recv().await {
					info!("收到来自{}的消息: {}", message.contact, message.message)
				}
			});
		}

		Ok(())
	}
}

#[puniyu_adapter::async_trait::async_trait]
impl AdapterApi for Adapter {
	fn adapter_info(&self) -> AdapterInfo {
		adapter_info!(
			name: NAME,
			version: VERSION,
			platform: AdapterPlatform::Other,
			standard: AdapterStandard::Other,
			protocol: AdapterProtocol::Console,
			communication: AdapterCommunication::Other,
		)
	}
	fn account_info(&self) -> AccountInfo {
		account_info!(
			uin: "console",
			name: format!("{}/{}", app_name(), "console"),
			avatar: get_logo(),
		)
	}
	async fn send_message(
		&self,
		contact: &ContactType<'_>,
		message: &Message,
	) -> puniyu_adapter::result::Result<SendMsgType> {
		if let Some(tx) = OUTPUT_TX.get() {
			let _ = tx.send(OutputMessage {
				contact: contact.peer().to_string(),
				message: format!("{:#?}", message),
			});
		}

		let message_id = make_random_id();
		let timestamp = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map_err(Box::<dyn std::error::Error + Send + Sync>::from)?
			.as_secs();

		Ok(SendMsgType { message_id, time: std::time::Duration::from_secs(timestamp) })
	}
}
