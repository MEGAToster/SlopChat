use tauri::{AppHandle, Manager, State};

use crate::config::ConfigState;
use crate::store::{chats, messages, search, Attachment, Chat, ChatView, Message, NewMessage, Store};

#[tauri::command(async)]
pub fn initial_chat(app: AppHandle, store: State<Store>) -> Result<Option<ChatView>, String> {
    let settings = app.state::<ConfigState>().get();
    chats::initial(&store.conn(), settings.startup, settings.resume_within_minutes)
}

#[tauri::command(async)]
pub fn list_chats(store: State<Store>) -> Result<Vec<Chat>, String> {
    chats::list(&store.conn())
}

#[tauri::command(async)]
pub fn create_chat(store: State<Store>, profile: String) -> Result<Chat, String> {
    chats::create(&store.conn(), &profile)
}

#[tauri::command(async)]
pub fn get_chat(store: State<Store>, chat_id: String) -> Result<Option<ChatView>, String> {
    chats::view_by_id(&store.conn(), &chat_id)
}

#[tauri::command(async)]
pub fn delete_chat(store: State<Store>, chat_id: String) -> Result<(), String> {
    chats::delete(&store.conn(), &chat_id)
}

#[tauri::command(async)]
pub fn toggle_chat_pin(store: State<Store>, chat_id: String) -> Result<Chat, String> {
    chats::toggle_pin(&store.conn(), &chat_id)
}

#[tauri::command(async)]
pub fn set_chat_profile(
    store: State<Store>,
    chat_id: String,
    profile: String,
) -> Result<(), String> {
    chats::set_profile(&store.conn(), &chat_id, &profile)
}

#[tauri::command(async)]
pub fn rename_chat(store: State<Store>, chat_id: String, title: String) -> Result<(), String> {
    chats::rename(&store.conn(), &chat_id, &title)
}

#[tauri::command(async)]
pub fn set_chat_model(
    store: State<Store>,
    chat_id: String,
    model: Option<String>,
) -> Result<(), String> {
    chats::set_model(&store.conn(), &chat_id, model.as_deref())
}

#[tauri::command(async)]
pub fn append_message(store: State<Store>, message: NewMessage) -> Result<Message, String> {
    messages::append(&store.conn(), &message)
}

#[tauri::command(async)]
pub fn rewind_to(
    store: State<Store>,
    chat_id: String,
    message_id: String,
    content: String,
) -> Result<(), String> {
    messages::rewind(&store.conn(), &chat_id, &message_id, &content)
}

#[tauri::command(async)]
pub fn search_chats(store: State<Store>, query: String) -> Result<Vec<Chat>, String> {
    search::chats(&store.conn(), &query, 20)
}

#[tauri::command(async)]
pub fn save_attachment(name: String, mime: String, data: String) -> Result<Attachment, String> {
    crate::store::attachments::save(&name, &mime, &data)
}

#[tauri::command(async)]
pub fn read_attachment(path: String) -> Result<String, String> {
    crate::store::attachments::read(&path)
}
