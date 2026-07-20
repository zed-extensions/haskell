use std::path::{Component, Path};

use zed::serde_json::{self, json};
use zed_extension_api::{
    self as zed, resolve_tcp_template, DebugAdapterBinary, DebugConfig, DebugRequest,
    DebugScenario, DebugTaskDefinition, Result, StartDebuggingRequestArguments,
    StartDebuggingRequestArgumentsRequest, TcpArguments, TcpArgumentsTemplate,
};

pub(crate) const HASKELL_DEBUG_ADAPTER: &str = "haskell-debugger";

fn merge_haskell_debug_configuration(
    task: &DebugTaskDefinition,
) -> Result<serde_json::Value, String> {
    let mut map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&task.config)
        .map_err(|err| {
            format!("Invalid Haskell debug configuration (expected JSON object): {err}")
        })?;
    map.entry("type".to_string())
        .or_insert(json!(HASKELL_DEBUG_ADAPTER));
    map.entry("request".to_string()).or_insert(json!("launch"));
    map.entry("name".to_string())
        .or_insert(json!(task.label.clone()));
    Ok(serde_json::Value::Object(map))
}

fn project_root_from_config(config: &serde_json::Value) -> Option<String> {
    config
        .get("projectRoot")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn resolve_hdb_binary(
    user_path: Option<String>,
    worktree: &zed::Worktree,
) -> Result<String, String> {
    if let Some(path) = user_path.filter(|p| !p.is_empty()) {
        return Ok(path);
    }
    worktree.which("hdb").ok_or_else(|| {
        "Could not find `hdb` on PATH. Install the Haskell debugger from https://well-typed.github.io/haskell-debugger/"
            .to_string()
    })
}

fn entry_path_relative_to_root(project_root: &str, program: &str) -> Result<String, String> {
    if program.is_empty() {
        return Err(
            "Program path is empty. Choose the Haskell entry file (e.g. app/Main.hs).".into(),
        );
    }
    let root = Path::new(project_root);
    let program_path = Path::new(program);
    let absolute_program = if program_path.is_absolute() {
        program_path.to_path_buf()
    } else {
        root.join(program_path)
    };
    let relative = absolute_program.strip_prefix(root).map_err(|_| {
        format!(
            "Program path `{}` must be inside project root `{}`",
            absolute_program.display(),
            project_root
        )
    })?;
    path_components_to_string(relative)
}

fn path_components_to_string(path: &Path) -> Result<String, String> {
    path.components()
        .filter_map(|c| match c {
            Component::Normal(part) => Some(Ok(part.to_string_lossy().into_owned())),
            Component::CurDir => None,
            Component::ParentDir => Some(Err("Program path must not contain `..`.".into())),
            _ => Some(Err("Invalid program path.".into())),
        })
        .collect::<Result<Vec<_>, _>>()
        .and_then(|parts| {
            if parts.is_empty() {
                Err("Resolved entry file path is empty.".into())
            } else {
                Ok(parts.join(std::path::MAIN_SEPARATOR_STR))
            }
        })
}

pub(crate) fn get_dap_binary(
    adapter_name: String,
    task: DebugTaskDefinition,
    user_provided_debug_adapter_path: Option<String>,
    worktree: &zed::Worktree,
) -> Result<DebugAdapterBinary, String> {
    if adapter_name != HASKELL_DEBUG_ADAPTER {
        return Err(format!(
            "Unknown Haskell debug adapter: {adapter_name} (expected {HASKELL_DEBUG_ADAPTER})"
        ));
    }

    let merged = merge_haskell_debug_configuration(&task)?;
    let request = dap_request_kind(adapter_name.as_str(), &merged)?;
    let configuration = serde_json::to_string(&merged)
        .map_err(|err| format!("Failed to serialize debug configuration: {err}"))?;

    let cwd = project_root_from_config(&merged).or_else(|| Some(worktree.root_path()));
    let envs = worktree.shell_env();

    if let Some(debug_server) = merged.get("debugServer").and_then(|v| v.as_u64()) {
        let port = u16::try_from(debug_server)
            .map_err(|_| format!("`debugServer` port must fit in 16 bits, got {debug_server}"))?;
        let tcp_connection = task.tcp_connection.unwrap_or(TcpArgumentsTemplate {
            host: None,
            port: Some(port),
            timeout: None,
        });
        let tcp = resolve_tcp_template(tcp_connection)?;
        return Ok(DebugAdapterBinary {
            command: None,
            arguments: vec![],
            envs,
            cwd,
            connection: Some(tcp),
            request_args: StartDebuggingRequestArguments {
                configuration,
                request,
            },
        });
    }

    let tcp_connection = task.tcp_connection.unwrap_or(TcpArgumentsTemplate {
        host: None,
        port: None,
        timeout: None,
    });
    let TcpArguments {
        host,
        port,
        timeout,
    } = resolve_tcp_template(tcp_connection)?;

    let command = resolve_hdb_binary(user_provided_debug_adapter_path, worktree)?;
    let mut arguments = vec!["server".to_string(), "--port".to_string(), port.to_string()];
    if merged
        .get("internalInterpreter")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        arguments.push("--internal-interpreter".to_string());
    }

    Ok(DebugAdapterBinary {
        command: Some(command),
        arguments,
        envs,
        cwd,
        connection: Some(TcpArguments {
            host,
            port,
            timeout,
        }),
        request_args: StartDebuggingRequestArguments {
            configuration,
            request,
        },
    })
}

pub(crate) fn dap_request_kind(
    adapter_name: &str,
    config: &serde_json::Value,
) -> Result<StartDebuggingRequestArgumentsRequest, String> {
    if adapter_name != HASKELL_DEBUG_ADAPTER {
        return Err(format!(
            "Unknown Haskell debug adapter: {adapter_name} (expected {HASKELL_DEBUG_ADAPTER})"
        ));
    }

    match config.get("request").and_then(|v| v.as_str()) {
        Some("attach") => Err(
            "The Haskell debugger (hdb) does not support attach; use request \"launch\".".into(),
        ),
        Some("launch") | None => Ok(StartDebuggingRequestArgumentsRequest::Launch),
        Some(other) => Err(format!(
            "Unsupported `request` for Haskell debugger: {other}"
        )),
    }
}

pub(crate) fn dap_config_to_scenario(config: DebugConfig) -> Result<DebugScenario, String> {
    if config.adapter != HASKELL_DEBUG_ADAPTER {
        return Err(format!(
            "This extension only defines the `{HASKELL_DEBUG_ADAPTER}` debug adapter (got {}).",
            config.adapter
        ));
    }

    match &config.request {
        DebugRequest::Attach(_) => Err(
            "The Haskell debugger (hdb) does not support attach in Zed; use a launch configuration."
                .into(),
        ),
        DebugRequest::Launch(launch) => {
            let project_root = launch.cwd.clone().ok_or_else(|| {
                "Set working directory to your Cabal/Stack project root; it becomes `projectRoot` for hdb."
                    .to_string()
            })?;
            let entry_file = entry_path_relative_to_root(&project_root, &launch.program)?;

            let body = json!({
                "projectRoot": project_root,
                "entryFile": entry_file,
                "entryPoint": "main",
                "entryArgs": launch.args.clone(),
                "extraGhcArgs": [],
            });

            Ok(DebugScenario {
                label: config.label,
                adapter: config.adapter,
                build: None,
                config: body.to_string(),
                tcp_connection: Some(TcpArgumentsTemplate {
                    host: None,
                    port: None,
                    timeout: None,
                }),
            })
        }
    }
}
