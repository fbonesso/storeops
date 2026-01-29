use clap::Subcommand;
use serde_json::{json, Value};

use crate::api::apple_client::AppleClient;

#[derive(Subcommand)]
pub enum TestflightCommand {
    /// Beta group management
    Groups {
        #[command(subcommand)]
        command: GroupsCommand,
    },
    /// Beta tester management
    Testers {
        #[command(subcommand)]
        command: TestersCommand,
    },
}

#[derive(Subcommand)]
pub enum GroupsCommand {
    /// List beta groups
    List {
        /// App ID
        app_id: String,
    },
    /// Create a beta group
    Create {
        /// App ID
        app_id: String,
        /// Group name
        #[arg(long)]
        name: String,
    },
}

#[derive(Subcommand)]
pub enum TestersCommand {
    /// List testers in a group
    List {
        /// Beta group ID
        group_id: String,
    },
    /// Add a tester to a group
    Add {
        /// Beta group ID
        group_id: String,
        /// Tester email
        #[arg(long)]
        email: String,
    },
}

pub async fn handle(
    cmd: &TestflightCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        TestflightCommand::Groups { command } => handle_groups(command, client, limit).await,
        TestflightCommand::Testers { command } => handle_testers(command, client, limit).await,
    }
}

async fn handle_groups(
    cmd: &GroupsCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        GroupsCommand::List { app_id } => {
            let mut query = vec![];
            let limit_str = limit.unwrap_or(50).to_string();
            query.push(("limit", limit_str.as_str()));
            query.push(("filter[app]", app_id.as_str()));
            client.get("/betaGroups", &query).await
        }
        GroupsCommand::Create { app_id, name } => {
            let body = json!({
                "data": {
                    "type": "betaGroups",
                    "attributes": {
                        "name": name
                    },
                    "relationships": {
                        "app": {
                            "data": {
                                "type": "apps",
                                "id": app_id
                            }
                        }
                    }
                }
            });
            client.post("/betaGroups", &body).await
        }
    }
}

async fn handle_testers(
    cmd: &TestersCommand,
    client: &AppleClient,
    limit: Option<u32>,
) -> Result<Value, Box<dyn std::error::Error>> {
    match cmd {
        TestersCommand::List { group_id } => {
            let mut query = vec![];
            let limit_str = limit.unwrap_or(50).to_string();
            query.push(("limit", limit_str.as_str()));
            client
                .get(
                    &format!("/betaGroups/{group_id}/betaTesters"),
                    &query,
                )
                .await
        }
        TestersCommand::Add { group_id, email } => {
            let tester_body = json!({
                "data": {
                    "type": "betaTesters",
                    "attributes": {
                        "email": email
                    },
                    "relationships": {
                        "betaGroups": {
                            "data": [
                                {
                                    "type": "betaGroups",
                                    "id": group_id
                                }
                            ]
                        }
                    }
                }
            });
            client.post("/betaTesters", &tester_body).await
        }
    }
}
