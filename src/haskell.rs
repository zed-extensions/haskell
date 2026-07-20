mod debugger;

use zed_extension_api::{
    self as zed,
    lsp::{Symbol, SymbolKind},
    settings::LspSettings,
    CodeLabel, CodeLabelSpan, DebugAdapterBinary, DebugConfig, DebugScenario, DebugTaskDefinition,
    Result, StartDebuggingRequestArgumentsRequest,
};

struct HaskellExtension;

impl zed::Extension for HaskellExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let lsp_settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;

        // If the user has specified a binary in their LSP settings,
        // that takes precedence.
        if let Some(binary_settings) = lsp_settings.binary {
            if let Some(path) = binary_settings.path {
                return Ok(zed::Command {
                    command: path,
                    args: binary_settings.arguments.unwrap_or_else(Vec::new),
                    env: worktree.shell_env(),
                });
            }
        }

        // Otherwise, default to hls installed via ghcup.
        let path = worktree
            .which("haskell-language-server-wrapper")
            .ok_or_else(|| "hls must be installed via ghcup".to_string())?;

        Ok(zed::Command {
            command: path,
            args: vec!["lsp".to_string()],
            env: worktree.shell_env(),
        })
    }

    fn language_server_workspace_configuration(
        &mut self,
        server_id: &zed::LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed_extension_api::serde_json::Value>> {
        Ok(LspSettings::for_worktree(server_id.as_ref(), worktree)
            .ok()
            .and_then(|s| s.settings))
    }

    fn label_for_symbol(
        &self,
        _language_server_id: &zed::LanguageServerId,
        symbol: Symbol,
    ) -> Option<CodeLabel> {
        let name = &symbol.name;

        let (code, display_range, filter_range) = match symbol.kind {
            SymbolKind::Struct => {
                let data_decl = "data ";
                let code = format!("{data_decl}{name} = A");
                let display_range = 0..data_decl.len() + name.len();
                let filter_range = data_decl.len()..display_range.end;
                (code, display_range, filter_range)
            }
            SymbolKind::Constructor => {
                let data_decl = "data A = ";
                let code = format!("{data_decl}{name}");
                let display_range = data_decl.len()..data_decl.len() + name.len();
                let filter_range = 0..name.len();
                (code, display_range, filter_range)
            }
            SymbolKind::Variable => {
                let code = format!("{name} :: T");
                let display_range = 0..name.len();
                let filter_range = 0..name.len();
                (code, display_range, filter_range)
            }
            _ => return None,
        };

        Some(CodeLabel {
            spans: vec![CodeLabelSpan::code_range(display_range)],
            filter_range: filter_range.into(),
            code,
        })
    }

    fn get_dap_binary(
        &mut self,
        adapter_name: String,
        task: DebugTaskDefinition,
        user_provided_debug_adapter_path: Option<String>,
        worktree: &zed::Worktree,
    ) -> Result<DebugAdapterBinary, String> {
        crate::debugger::get_dap_binary(
            adapter_name,
            task,
            user_provided_debug_adapter_path,
            worktree,
        )
    }

    fn dap_request_kind(
        &mut self,
        adapter_name: String,
        config: zed::serde_json::Value,
    ) -> Result<StartDebuggingRequestArgumentsRequest, String> {
        crate::debugger::dap_request_kind(&adapter_name, &config)
    }

    fn dap_config_to_scenario(&mut self, config: DebugConfig) -> Result<DebugScenario, String> {
        crate::debugger::dap_config_to_scenario(config)
    }
}

zed::register_extension!(HaskellExtension);
