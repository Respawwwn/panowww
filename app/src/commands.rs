//! The Tauri commands: the only surface the UI calls. Every type crossing
//! this boundary derives `TS` and is exported to `ui/src/lib/generated/`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The answer to `ping`: proves the IPC and shares the app version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Pong {
    pub app: String,
    pub version: String,
}

#[tauri::command]
pub fn ping() -> Pong {
    Pong {
        app: "Panowww".to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
    }
}

/// Every command, registered on the builder.
pub fn handler<R: tauri::Runtime>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static
{
    tauri::generate_handler![ping]
}

#[cfg(test)]
mod tests {
    use tauri::WebviewWindowBuilder;
    use tauri::test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder};
    use tauri::webview::InvokeRequest;

    use super::*;

    fn app() -> tauri::App<MockRuntime> {
        mock_builder()
            .invoke_handler(handler())
            .build(crate::context())
            .expect("mock app builds")
    }

    #[test]
    fn ping_answers_with_the_workspace_version() {
        let app = app();
        let webview = WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::default())
            .build()
            .expect("mock window builds");
        let response = get_ipc_response(
            &webview,
            InvokeRequest {
                cmd: "ping".to_owned(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                // The app origin differs per OS, as the ACL expects.
                url: if cfg!(windows) {
                    "http://tauri.localhost"
                } else {
                    "tauri://localhost"
                }
                .parse()
                .unwrap(),
                body: tauri::ipc::InvokeBody::default(),
                headers: tauri::http::HeaderMap::default(),
                invoke_key: INVOKE_KEY.to_owned(),
            },
        )
        .expect("ping succeeds");
        let pong: Pong = response.deserialize().unwrap();
        assert_eq!(pong.app, "Panowww");
        assert_eq!(pong.version, env!("CARGO_PKG_VERSION"));
    }
}
