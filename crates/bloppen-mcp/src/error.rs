//! Turning an API failure into a tool result a caller can act on.

use bloppen_api::Error;
use rmcp::model::{CallToolResult, ContentBlock};

/// Report a failed call.
pub fn failed(error: &Error) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message(error))])
}

fn message(error: &Error) -> String {
    if error.is_auth_expired() {
        return "The Børneloppen session is not authenticated and could not be renewed. \
                Set BORNELOPPEN_EMAIL and BORNELOPPEN_PASSWORD in the server environment, \
                or run the CLI's `auth login` to populate the session file."
            .to_owned();
    }

    match error {
        Error::Auth(bloppen_api::AuthError::MissingCredentials) => {
            "No session and no credentials. Set BORNELOPPEN_EMAIL and BORNELOPPEN_PASSWORD \
             in the server environment so the client can sign in."
                .to_owned()
        }
        Error::Decode { .. } => {
            "Børneloppen answered with a body this client could not decode. The site may be \
             serving an error page instead of the expected JSON."
                .to_owned()
        }
        Error::Transport(_) => format!("Could not reach Børneloppen: {error}"),
        other => other.to_string(),
    }
}
