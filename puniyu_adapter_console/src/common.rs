use crate::input::{ConsolePayload, ParsedConsoleInput};
use puniyu_adapter::event::{
	crate_friend_message, crate_group_message, crate_group_temp_message, crate_guild_message,
	create_event, create_message, send_event,
};
use puniyu_adapter::contact::*;
use puniyu_adapter::element::receive::*;
use puniyu_adapter::sender::*;
use puniyu_adapter::bot::Bot;
use rand::distr::{Alphanumeric, SampleString};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn make_random_id() -> String {
	Alphanumeric.sample_string(&mut rand::rng(), 32)
}

const DEFAULT_GUILD_ID: &str = "test_guild";
const DEFAULT_GUILD_NAME: &str = "test_guild";
const DEFAULT_GUILD_SUB_NAME: &str = "test_channel";

macro_rules! dispatch_message {
	(Friend, $bot:expr, $event_id:expr, $time:expr, $msg_id:expr, $elements:expr) => {{
		let bot_name = $bot.self_id();
		let contact = contact_friend!(bot_name, bot_name);
		let sender = sender_friend!(user_id: bot_name, nick: bot_name);
		let event = create_event!(
			Message,
			create_message!(Friend, crate_friend_message!(
				bot: $bot,
				event_id: $event_id,
				user_id: bot_name,
				contact: &contact,
				sender: &sender,
				time: $time,
				message_id: $msg_id,
				elements: $elements,
			))
		);
		send_event(event).await;
	}};
	(Group, $bot:expr, $event_id:expr, $time:expr, $msg_id:expr, $elements:expr) => {{
		let bot_name = $bot.self_id();
		let contact = contact_group!(bot_name, bot_name);
		let sender = sender_group!(user_id: bot_name, nick: bot_name, sex: Sex::Unknown, age: 0, role: Role::Member);
		let event = create_event!(
			Message,
			create_message!(Group, crate_group_message!(
				bot: $bot,
				event_id: $event_id,
				user_id: bot_name,
				contact: &contact,
				sender: &sender,
				time: $time,
				message_id: $msg_id,
				elements: $elements,
			))
		);
		send_event(event).await;
	}};
	(GroupTemp, $bot:expr, $event_id:expr, $time:expr, $msg_id:expr, $elements:expr) => {{
		let bot_name = $bot.self_id();
		let contact = contact_group_temp!(bot_name, bot_name);
		let sender = sender_group_temp!(user_id: bot_name, nick: bot_name, sex: Sex::Unknown, age: 0, role: Role::Member);
		let event = create_event!(
			Message,
			create_message!(GroupTemp, crate_group_temp_message!(
				bot: $bot,
				event_id: $event_id,
				user_id: bot_name,
				contact: &contact,
				sender: &sender,
				time: $time,
				message_id: $msg_id,
				elements: $elements,
			))
		);
		send_event(event).await;
	}};
	(Guild, $bot:expr, $event_id:expr, $time:expr, $msg_id:expr, $elements:expr) => {{
		let bot_name = $bot.self_id();
		let contact = contact_guild!(peer: DEFAULT_GUILD_ID, name: DEFAULT_GUILD_NAME, sub_name: DEFAULT_GUILD_SUB_NAME);
		let sender = sender_guild!(user_id: bot_name, nick: bot_name, sex: Sex::Unknown, age: 0, role: Role::Member);
		let event = create_event!(
			Message,
			create_message!(Guild, crate_guild_message!(
				bot: $bot,
				event_id: $event_id,
				user_id: bot_name,
				contact: &contact,
				sender: &sender,
				time: $time,
				message_id: $msg_id,
				elements: $elements,
			))
		);
		send_event(event).await;
	}};
}

pub async fn dispatch_event(bot: &Bot, input: &ParsedConsoleInput) {
	let elements = build_elements(&input.payload);
	let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
	let message_id = make_random_id();
	let event_id = make_random_id();

	match input.scene {
		SceneType::Friend => dispatch_message!(Friend, bot, &event_id, timestamp, &message_id, &elements),
		SceneType::Group => dispatch_message!(Group, bot, &event_id, timestamp, &message_id, &elements),
		SceneType::GroupTemp => dispatch_message!(GroupTemp, bot, &event_id, timestamp, &message_id, &elements),
		SceneType::Guild => dispatch_message!(Guild, bot, &event_id, timestamp, &message_id, &elements),
	}
}

fn build_elements(payload: &ConsolePayload) -> Vec<Elements<'_>> {
	match payload {
		ConsolePayload::At(target_id) => vec![Elements::At(AtElement { target_id })],
		ConsolePayload::Text(text) => vec![Elements::Text(TextElement { text })],
		ConsolePayload::Image(image_url) => vec![Elements::Image(ImageElement {
			file: image_url.as_str().as_bytes().to_vec().into(),
			file_name: "image.png",
			summary: "image",
			width: 100,
			height: 100,
		})],
		ConsolePayload::Json(json_content) => {
			vec![Elements::Json(JsonElement { data: json_content })]
		}
		ConsolePayload::Video(video_url) => vec![Elements::Video(VideoElement {
			file: video_url.as_str().as_bytes().to_vec().into(),
			file_name: "video",
		})],
		ConsolePayload::Record(record_url) => vec![Elements::Record(RecordElement {
			file: record_url.as_str().as_bytes().to_vec().into(),
			file_name: "record",
		})],
		ConsolePayload::File(file_url) => vec![Elements::File(FileElement {
			file: file_url.as_str().as_bytes().to_vec().into(),
			file_size: 0,
			file_name: "file",
		})],
		ConsolePayload::Xml(xml_content) => vec![Elements::Xml(XmlElement { data: xml_content })],
	}
}
