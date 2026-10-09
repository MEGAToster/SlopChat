    use super::*;
    use crate::config::Startup;

    fn message(chat_id: &str, role: &str, content: &str) -> NewMessage {
        NewMessage {
            chat_id: chat_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            model: None,
            tokens_in: None,
            tokens_out: None,
            cost: None,
            provider: None,
            sources: None,
            attachments: Vec::new(),
        }
    }

    fn list_ids(conn: &rusqlite::Connection) -> Vec<String> {
        chats::list(conn)
            .unwrap()
            .into_iter()
            .map(|chat| chat.id)
            .collect()
    }

    #[test]
    fn migrations_create_a_working_schema() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let count: i64 = conn
            .query_row("SELECT count(*) FROM messages_fts", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn appending_messages_indexes_them_for_search() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = chats::create(&conn, "default").unwrap();
        messages::append(&conn, &message(&chat.id, "user", "hello world")).unwrap();

        assert_eq!(messages::list(&conn, &chat.id).unwrap().len(), 1);
        let hits: i64 = conn
            .query_row(
                "SELECT count(*) FROM messages_fts WHERE messages_fts MATCH 'hello'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(hits, 1);

        let chat = chats::get(&conn, &chat.id).unwrap().unwrap();
        assert_eq!(chat.title, "hello world");
    }

    #[test]
    fn appending_messages_round_trips_the_provider() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = chats::create(&conn, "default").unwrap();

        let mut new = message(&chat.id, "assistant", "hi");
        new.provider = Some("DeepInfra".to_string());
        messages::append(&conn, &new).unwrap();

        let listed = messages::list(&conn, &chat.id).unwrap();
        assert_eq!(listed[0].provider.as_deref(), Some("DeepInfra"));
    }

    #[test]
    fn pinning_is_exclusive() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let first = chats::create(&conn, "default").unwrap();
        let second = chats::create(&conn, "default").unwrap();

        chats::toggle_pin(&conn, &first.id).unwrap();
        chats::toggle_pin(&conn, &second.id).unwrap();

        assert_eq!(chats::pinned(&conn).unwrap().unwrap().id, second.id);
        assert!(!chats::get(&conn, &first.id).unwrap().unwrap().pinned);
    }

    #[test]
    fn resume_modes_pick_the_right_chat() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let first = chats::create(&conn, "default").unwrap();
        let second = chats::create(&conn, "default").unwrap();
        chats::touch(&conn, &first.id, now_millis() - 60_000).unwrap();
        chats::touch(&conn, &second.id, now_millis()).unwrap();

        assert!(chats::initial(&conn, Startup::New, 10).unwrap().is_none());
        assert_eq!(
            chats::initial(&conn, Startup::Latest, 10).unwrap().unwrap().chat.id,
            second.id
        );

        // Pinned only opens a chat when one is pinned.
        assert!(chats::initial(&conn, Startup::Pinned, 10).unwrap().is_none());
        chats::toggle_pin(&conn, &first.id).unwrap();
        assert_eq!(
            chats::initial(&conn, Startup::Pinned, 10).unwrap().unwrap().chat.id,
            first.id
        );
        // Latest ignores the pin.
        assert_eq!(
            chats::initial(&conn, Startup::Latest, 10).unwrap().unwrap().chat.id,
            second.id
        );
    }

    #[test]
    fn latest_staleness_zero_means_always() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = chats::create(&conn, "default").unwrap();
        chats::touch(&conn, &chat.id, now_millis() - 11 * 60_000).unwrap();

        assert!(chats::initial(&conn, Startup::Latest, 10).unwrap().is_none());
        assert_eq!(
            chats::initial(&conn, Startup::Latest, 0).unwrap().unwrap().chat.id,
            chat.id
        );
    }

    #[test]
    fn rewind_drops_later_messages() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = chats::create(&conn, "default").unwrap();
        let first = messages::append(&conn, &message(&chat.id, "user", "one")).unwrap();
        messages::append(&conn, &message(&chat.id, "assistant", "two")).unwrap();
        messages::append(&conn, &message(&chat.id, "user", "three")).unwrap();

        messages::rewind(&conn, &chat.id, &first.id, "one edited").unwrap();

        let listed = messages::list(&conn, &chat.id).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].content, "one edited");
    }

    #[test]
    fn list_orders_by_last_edit() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let first = chats::create(&conn, "p").unwrap();
        let second = chats::create(&conn, "p").unwrap();

        chats::touch(&conn, &first.id, 1_000).unwrap();
        chats::touch(&conn, &second.id, 2_000).unwrap();
        assert_eq!(list_ids(&conn), vec![first.id.clone(), second.id.clone()]);

        chats::touch(&conn, &first.id, 3_000).unwrap();
        assert_eq!(list_ids(&conn), vec![second.id.clone(), first.id.clone()]);
    }

    #[test]
    fn rewind_moves_its_chat_to_the_end() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let first = chats::create(&conn, "p").unwrap();
        let second = chats::create(&conn, "p").unwrap();
        let edited = messages::append(&conn, &message(&first.id, "user", "one")).unwrap();

        // Park both at the same old time so created order decides.
        chats::touch(&conn, &first.id, 0).unwrap();
        chats::touch(&conn, &second.id, 0).unwrap();
        assert_eq!(list_ids(&conn), vec![first.id.clone(), second.id.clone()]);

        messages::rewind(&conn, &first.id, &edited.id, "edited").unwrap();
        assert_eq!(list_ids(&conn), vec![second.id.clone(), first.id.clone()]);
    }

    #[test]
    fn rewind_of_unknown_message_errors() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = chats::create(&conn, "default").unwrap();

        assert!(messages::rewind(&conn, &chat.id, "missing", "x").is_err());
    }

    #[test]
    fn renaming_a_profile_moves_existing_chats() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = chats::create(&conn, "old").unwrap();

        chats::rename_profile(&conn, "old", "new").unwrap();

        assert_eq!(
            chats::get(&conn, &chat.id).unwrap().unwrap().profile,
            "new"
        );
    }

    #[test]
    fn renaming_a_chat_sets_its_title() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = chats::create(&conn, "default").unwrap();

        chats::rename(&conn, &chat.id, "  My title  ").unwrap();

        assert_eq!(chats::get(&conn, &chat.id).unwrap().unwrap().title, "My title");
    }

    #[test]
    fn renaming_a_chat_rejects_an_empty_title() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = chats::create(&conn, "default").unwrap();

        assert!(chats::rename(&conn, &chat.id, "   ").is_err());
    }

    #[test]
    fn renaming_an_unknown_chat_errors() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();

        assert!(chats::rename(&conn, "missing", "title").is_err());
    }

    #[test]
    fn chat_model_round_trips_and_clears() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = chats::create(&conn, "p").unwrap();
        assert_eq!(chats::get(&conn, &chat.id).unwrap().unwrap().model, None);

        chats::set_model(&conn, &chat.id, Some("openrouter/a")).unwrap();
        assert_eq!(
            chats::get(&conn, &chat.id).unwrap().unwrap().model.as_deref(),
            Some("openrouter/a")
        );

        chats::set_model(&conn, &chat.id, None).unwrap();
        assert_eq!(chats::get(&conn, &chat.id).unwrap().unwrap().model, None);
    }

    #[test]
    fn agy_session_round_trips_and_clears() {
        let store = Store::in_memory().unwrap();
        let conn = store.conn();
        let chat = chats::create(&conn, "p").unwrap();
        assert_eq!(chats::get_agy_session(&conn, &chat.id).unwrap(), None);

        chats::set_agy_session(&conn, &chat.id, Some("conv")).unwrap();
        assert_eq!(
            chats::get_agy_session(&conn, &chat.id).unwrap().as_deref(),
            Some("conv")
        );

        chats::set_agy_session(&conn, &chat.id, None).unwrap();
        assert_eq!(chats::get_agy_session(&conn, &chat.id).unwrap(), None);
    }
