# aegisrlib

![License](https://img.shields.io/badge/license-MIT-blue.svg) ![Version](https://img.shields.io/badge/version-1.0.0-green.svg) ![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)

The core library dependency for **AegisR** daemon and terminal. `aegisrlib` provides a robust backend for filesystem management, collection-based key-value storage, and threaded persistence.

## Features

- **Filesystem & Config**: Automatic path resolution and initialization.
- **Collection System**: Support for multiple isolated data contexts.
- **CRUD Operations**: Fast `put`, `get`, `delete`, and `clear` methods.
- **Persistence**: Background auto-save threads and manual flush controls.

## Installation

To use `aegisrlib` in your Rust project, add the following to your `Cargo.toml`:

```toml
[dependencies]
aegisrlib = { git = "https://github.com/surelle-ha/aegisr", branch="main" }
```

For development or testing releases, change the branch to the desired version, e.g., `branch="1.0.2-development"`. Please note that unrelease versions may have breaking changes and untested features.

```toml
[dependencies]
aegisrlib = { git = "https://github.com/surelle-ha/aegisr", branch="1.0.1-development" }
```

## Usage

Here is a complete rundown based on the runtime demo:

```rust
// 0. Ensure required modules are imported
use aegisrlib::{AegCore, AegFileSystem};

fn main() {
    println!("=======================================");
    println!("   🚀 AEGISRLIB RUNTIME FEATURE DEMO");
    println!("=======================================\n");

    // --- SETUP & ENGINE LOAD ---
    println!("[0] ⚙️ Initializing Filesystem and Configuration...");
    // Initialize config, suppressing output but forcing creation (true, true)
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

    // ------------------------------------
    //        CRUD OPERATIONS DEMO
    // ------------------------------------
    println!("[3] 🔑 Key-Value CRUD Operations (in active collection '{}')", engine.active_collection);

    println!("\n[3.1] **CREATE/PUT** Operation Demonstration...");
    let put_result = AegCore::put_value("greeting", "hello world");
    println!("  > PUT 'greeting' = 'hello world' | Result: {:?}", put_result);

    let val = AegCore::get_value("greeting").unwrap_or("NOT FOUND".into());
    println!("  > GET 'greeting' => **{}**", val);

    println!("\n[3.2] **UPDATE** Demonstration...");
    AegCore::put_value("greeting", "new value");
    println!("  > PUT (update) 'greeting' = 'new value'");
    println!("  > GET 'greeting' (updated) => **{:?}**", AegCore::get_value("greeting"));

    println!("\n[3.3] **DELETE** Demonstration...");
    AegCore::delete_value("greeting");
    let val_after_delete = AegCore::get_value("greeting").unwrap_or("**NOT FOUND**".into());
    println!("  > DELETE 'greeting'");
    println!("  > GET 'greeting' after delete => {}", val_after_delete);

    println!("\n[3.4] Multiple Key Insertion & Retrieval...");
    AegCore::put_value("username", "harold");
    AegCore::put_value("password", "super_secret");
    AegCore::put_value("role", "admin");
    println!("  > Inserted 'username', 'password', and 'role'.");

    println!("  Reading back values:");
    println!("    * username => {}", AegCore::get_value("username").unwrap());
    println!("    * password => {}", AegCore::get_value("password").unwrap());
    println!("    * role     => {}", AegCore::get_value("role").unwrap());
    println!("  ✅ Multiple read successful.");

    println!("\n[3.5] **CLEAR** All Values Demonstration...");
    AegCore::clear_values();
    println!("  > CLEAR all values in active collection.");

    println!("  Trying to read 'username' after CLEAR...");
    println!(
        "  > GET 'username' => {}\n",
        AegCore::get_value("username").unwrap_or("**NOT FOUND**".into())
    );

    // ------------------------------------
    //        COLLECTION SWITCH DEMO
    // ------------------------------------
    println!("[3.6] 📂 Multi-Collection Management Demonstration");

    let new_collection = "secondary";
    println!("  > Creating new collection: **'{}'**", new_collection);
    let create_msg = AegCore::create_collection(new_collection);
    println!("  > Creation Result: {}", create_msg);

    // Reload engine to recognize the new collection if necessary
    engine = AegCore::load();
    println!("  > Switching active collection to **'{}'**", new_collection);
    match engine.set_active_collection(new_collection) {
        Ok(_) => println!("  ✅ Active collection is now: **'{}'**", engine.active_collection),
        Err(e) => println!("  ❌ Failed to switch: {}", e),
    }

    println!("  > Adding session values in the new collection...");
    AegCore::put_value("session_token", "abcd1234");
    AegCore::put_value("user_email", "harold@example.com");

    println!("  Reading values from **'{}'**:", new_collection);
    println!("    * session_token => {:?}", AegCore::get_value("session_token"));
    println!("    * user_email    => {:?}", AegCore::get_value("user_email"));

    println!("  > Switching back to **'default'** collection...");
    match engine.set_active_collection("default") {
        Ok(_) => println!("  ✅ Active collection is now: **'{}'**", engine.active_collection),
        Err(e) => println!("  ❌ Failed to switch: {}", e),
    }

    println!("  Reading 'username' from 'default' (should be NOT FOUND due to previous CLEAR):");
    println!("    * username => {:?}", AegCore::get_value("username"));

    // ------------------------------------
    //        CLEANUP & SHUTDOWN
    // ------------------------------------
    println!("\n[4] 🛑 Engine Shutdown Sequence");

    println!("[4.1] Manual final save before shutdown...");
    engine.save();
    println!("  ✅ Manual engine save completed.\n");

    println!("[4.2] Stopping automatic background saver...");
    AegCore::stop_background_saver();
    println!("  ✅ Background saver stopped.\n");

    println!("[4.3] **FLUSH** NOW (Forces immediate write to disk)...");
    AegCore::flush_now();
    println!("  ✅ Data fully flushed.\n");

    println!("=======================================");
    println!("     ✨ DEMO EXECUTION COMPLETE ✨");
    println!("=======================================");
}
```
