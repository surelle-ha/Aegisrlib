use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};

// INIT
#[derive(Args, Debug)]
pub struct InitArgs {
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,
    #[arg(short, long, help = "Reset configuration files")]
    pub reset: bool,
}

// USE
#[derive(Args, Debug)]
pub struct UseArgs {
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,
    #[arg(help = "Name of the collection to activate")]
    pub name: String,
}

// NEW
#[derive(Args, Debug)]
pub struct NewArgs {
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,
    #[arg(help = "Name of the new collection to create")]
    pub name: String,
}

// DELETE
#[derive(Args, Debug)]
pub struct DeleteArgs {
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,
    #[arg(help = "Name of the collection to delete")]
    pub name: String,
}

// RENAME
#[derive(Args, Debug)]
pub struct RenameArgs {
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,
    #[arg(help = "Name of the collection to rename")]
    pub name: String,
    #[arg(help = "New name for the collection")]
    pub new_name: String,
}

#[derive(Args, Debug)]
pub struct PutArgs {
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,
    #[arg(help = "Key to store in the active collection")]
    pub key: String,
    #[arg(help = "Value to associate with the key")]
    pub value: String,
}

#[derive(Args, Debug)]
pub struct GetArgs {
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,
    #[arg(help = "Key to retrieve from the active collection")]
    pub key: String,
}

#[derive(Args, Debug)]
pub struct DelArgs {
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,
    #[arg(help = "Key to delete from the active collection")]
    pub key: String,
}

#[derive(Args, Debug)]
pub struct ClearArgs {
    #[arg(short, long, help = "Enable verbose output")]
    pub verbose: bool,
}

// ===========================
// SUBCOMMAND ENUM
// ===========================

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Initialize the configuration")]
    Init(InitArgs),
    #[command(about = "List all collections")]
    List,
    #[command(about = "Switch to a different collection")]
    Use(UseArgs),
    #[command(about = "Create a new collection")]
    New(NewArgs),
    #[command(about = "Delete an existing collection")]
    Delete(DeleteArgs),
    #[command(about = "Rename an existing collection")]
    Rename(RenameArgs),
    #[command(about = "Show the current status")]
    Status,
    #[command(about = "Store a key/value pair in the active collection")]
    Put(PutArgs),
    #[command(about = "Retrieve the value of a key from the active collection")]
    Get(GetArgs),
    #[command(about = "Delete a key/value pair from the active collection")]
    Del(DelArgs),
    #[command(about = "Clear all key/value pairs from the active collection")]
    Clear(ClearArgs),
}

// ===========================
// AegisrCommand ENUM
// ===========================

#[derive(Serialize, Deserialize, Debug)]
pub enum AegisrCommand {
    Init { verbose: bool, reset: bool },
    List,
    Use { verbose: bool, name: String },
    New { verbose: bool, name: String },
    Delete { verbose: bool, name: String },
    Rename { verbose: bool, name: String, new_name: String },
    Status,
    Put { verbose: bool, key: String, value: String },
    Get { verbose: bool, key: String },
    Del { verbose: bool, key: String },
    Clear { verbose: bool },
}
