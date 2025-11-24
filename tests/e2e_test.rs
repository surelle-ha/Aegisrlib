use aegisrlib::{AegCore, AegFileSystem};

#[test]
fn e2e_test() {
    println!("=======================================");
    println!("   🚀 AEGISRLIB RUNTIME FEATURE DEMO");
    println!("=======================================\n");

    println!("[0] ⚙️ Initializing Filesystem and Configuration...");
    let config_path = AegFileSystem::initialize_config(Some(false), Some(true));
    println!("  ✅ Config initialized at: {:?}\n", config_path);

    println!("[1] 💾 Loading Engine from Storage...");
    let mut engine: AegCore = AegCore::load();

    if engine.collections.is_empty() {
        println!("  ⚠️ No existing collections found → Creating 'default' collection...");
        engine.collections.push("default".to_string());
    } else {
        println!("  ✅ Existing collections found: {:?}", engine.collections);
    }

    if engine.active_collection.is_empty() {
        println!("  ⚠️ No active collection set → Auto-selecting 'default'");
        engine.active_collection = engine.collections[0].clone();
    }

    println!("  ✨ Active collection is now: '{}'\n", engine.active_collection);

    println!("[1.1] 📝 Persisting updated engine metadata...");
    engine.save();
    println!("  ✅ Engine metadata saved.\n");

    println!("[2] ⏱️ Starting automatic background saver (interval: 60s)...");
    AegCore::start_background_saver(60);
    println!("  ✅ Background saver is now running.\n");

    println!("[3] 🔑 Key-Value CRUD Operations...");
    AegCore::put_value("greeting", "hello world");
    assert_eq!(AegCore::get_value("greeting").unwrap(), "hello world");

    AegCore::put_value("greeting", "new value");
    assert_eq!(AegCore::get_value("greeting").unwrap(), "new value");

    AegCore::delete_value("greeting");
    assert!(AegCore::get_value("greeting").is_none());

    AegCore::put_value("username", "harold");
    AegCore::put_value("password", "super_secret");
    AegCore::put_value("role", "admin");

    assert_eq!(AegCore::get_value("username").unwrap(), "harold");
    assert_eq!(AegCore::get_value("password").unwrap(), "super_secret");
    assert_eq!(AegCore::get_value("role").unwrap(), "admin");

    AegCore::clear_values();
    assert!(AegCore::get_value("username").is_none());

    println!("[3.6] 📂 Multi-Collection Management...");
    let new_collection = "secondary";
    AegCore::create_collection(new_collection);

    engine = AegCore::load();
    engine.set_active_collection(new_collection).unwrap();

    AegCore::put_value("session_token", "abcd1234");
    AegCore::put_value("user_email", "harold@example.com");

    assert_eq!(AegCore::get_value("session_token").unwrap(), "abcd1234");
    assert_eq!(AegCore::get_value("user_email").unwrap(), "harold@example.com");

    engine.set_active_collection("default").unwrap();
    assert!(AegCore::get_value("username").is_none());

    engine.save();
    AegCore::stop_background_saver();
    AegCore::flush_now();

    println!("=======================================");
    println!("     ✨ USAGE DEMO TEST COMPLETE ✨");
    println!("=======================================");
}
