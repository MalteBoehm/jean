use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::projects::storage::load_projects_data;
use crate::MagicPromptBackends;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    #[default]
    Claude,
    Codex,
    Opencode,
}

impl AiProvider {
    fn from_backend_str(raw: Option<&str>) -> Self {
        match raw.unwrap_or("claude") {
            "codex" => Self::Codex,
            "opencode" => Self::Opencode,
            _ => Self::Claude,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderCapabilities {
    pub chat: bool,
    pub session_naming: bool,
    pub branch_naming: bool,
    pub pr_content_generation: bool,
    pub commit_message_generation: bool,
    pub context_summarization: bool,
    pub session_recap: bool,
    pub code_review: bool,
    pub mcp: bool,
    pub custom_profiles: bool,
    pub usage_reporting: bool,
    pub thinking_controls: bool,
    pub chrome_integration: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderStatus {
    pub installed: bool,
    pub authenticated: bool,
    pub available: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub capabilities: AiProviderCapabilities,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderStatuses {
    pub claude: AiProviderStatus,
    pub codex: AiProviderStatus,
    pub opencode: AiProviderStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedFeatureProviders {
    pub chat: AiProvider,
    pub investigate_issue: AiProvider,
    pub investigate_pr: AiProvider,
    pub investigate_workflow_run: AiProvider,
    pub investigate_security_alert: AiProvider,
    pub investigate_advisory: AiProvider,
    pub investigate_linear_issue: AiProvider,
    pub pr_content_generation: AiProvider,
    pub commit_message_generation: AiProvider,
    pub code_review: AiProvider,
    pub context_summarization: AiProvider,
    pub resolve_conflicts: AiProvider,
    pub release_notes: AiProvider,
    pub session_naming: AiProvider,
    pub branch_naming: AiProvider,
    pub session_recap: AiProvider,
    pub mcp: AiProvider,
    pub custom_profiles: AiProvider,
    pub usage_reporting: AiProvider,
    pub thinking_controls: AiProvider,
    pub chrome_integration: AiProvider,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderOverview {
    pub default_chat_provider: AiProvider,
    pub providers: AiProviderStatuses,
    pub selected_feature_providers: SelectedFeatureProviders,
}

#[derive(Debug, Clone, Default)]
struct ProviderPreferenceSnapshot {
    default_backend: String,
    magic_prompt_backends: MagicPromptBackends,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProviderFeature {
    Chat,
    InvestigateIssue,
    InvestigatePr,
    InvestigateWorkflowRun,
    InvestigateSecurityAlert,
    InvestigateAdvisory,
    InvestigateLinearIssue,
    PrContentGeneration,
    CommitMessageGeneration,
    CodeReview,
    ContextSummarization,
    ResolveConflicts,
    ReleaseNotes,
    SessionNaming,
    BranchNaming,
    SessionRecap,
    Mcp,
    CustomProfiles,
    UsageReporting,
    ThinkingControls,
    ChromeIntegration,
}

fn load_preference_snapshot(app: &AppHandle) -> ProviderPreferenceSnapshot {
    crate::get_preferences_path(app)
        .ok()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|contents| serde_json::from_str::<crate::AppPreferences>(&contents).ok())
        .map(|prefs| ProviderPreferenceSnapshot {
            default_backend: prefs.default_backend,
            magic_prompt_backends: prefs.magic_prompt_backends,
        })
        .unwrap_or_else(|| ProviderPreferenceSnapshot {
            default_backend: "claude".to_string(),
            magic_prompt_backends: MagicPromptBackends::default(),
        })
}

fn load_project_default_backend(app: &AppHandle, worktree_id: Option<&str>) -> Option<String> {
    let worktree_id = worktree_id?;
    let data = load_projects_data(app).ok()?;
    let worktree = data
        .worktrees
        .iter()
        .find(|worktree| worktree.id == worktree_id)?;
    data.projects
        .iter()
        .find(|project| !project.is_folder && project.id == worktree.project_id)
        .and_then(|project| project.default_backend.clone())
}

fn resolve_feature_override(
    magic_prompt_backends: &MagicPromptBackends,
    feature: ProviderFeature,
) -> Option<&str> {
    match feature {
        ProviderFeature::InvestigateIssue => {
            magic_prompt_backends.investigate_issue_backend.as_deref()
        }
        ProviderFeature::InvestigatePr => magic_prompt_backends.investigate_pr_backend.as_deref(),
        ProviderFeature::InvestigateWorkflowRun => magic_prompt_backends
            .investigate_workflow_run_backend
            .as_deref(),
        ProviderFeature::InvestigateSecurityAlert => magic_prompt_backends
            .investigate_security_alert_backend
            .as_deref(),
        ProviderFeature::InvestigateAdvisory => magic_prompt_backends
            .investigate_advisory_backend
            .as_deref(),
        ProviderFeature::InvestigateLinearIssue => magic_prompt_backends
            .investigate_linear_issue_backend
            .as_deref(),
        ProviderFeature::PrContentGeneration => magic_prompt_backends.pr_content_backend.as_deref(),
        ProviderFeature::CommitMessageGeneration => {
            magic_prompt_backends.commit_message_backend.as_deref()
        }
        ProviderFeature::CodeReview => magic_prompt_backends.code_review_backend.as_deref(),
        ProviderFeature::ContextSummarization => {
            magic_prompt_backends.context_summary_backend.as_deref()
        }
        ProviderFeature::ResolveConflicts => {
            magic_prompt_backends.resolve_conflicts_backend.as_deref()
        }
        ProviderFeature::ReleaseNotes => magic_prompt_backends.release_notes_backend.as_deref(),
        ProviderFeature::SessionNaming | ProviderFeature::BranchNaming => {
            magic_prompt_backends.session_naming_backend.as_deref()
        }
        ProviderFeature::SessionRecap => magic_prompt_backends.session_recap_backend.as_deref(),
        ProviderFeature::Chat
        | ProviderFeature::Mcp
        | ProviderFeature::CustomProfiles
        | ProviderFeature::UsageReporting
        | ProviderFeature::ThinkingControls
        | ProviderFeature::ChromeIntegration => None,
    }
}

fn resolve_provider_for_feature(
    default_backend: &str,
    project_default_backend: Option<&str>,
    magic_prompt_backends: &MagicPromptBackends,
    feature: ProviderFeature,
) -> AiProvider {
    let feature_backend =
        resolve_feature_override(magic_prompt_backends, feature).filter(|value| !value.is_empty());
    AiProvider::from_backend_str(
        feature_backend
            .or(project_default_backend)
            .or(Some(default_backend)),
    )
}

fn build_selected_feature_providers(
    default_backend: &str,
    project_default_backend: Option<&str>,
    magic_prompt_backends: &MagicPromptBackends,
) -> SelectedFeatureProviders {
    let resolve = |feature| {
        resolve_provider_for_feature(
            default_backend,
            project_default_backend,
            magic_prompt_backends,
            feature,
        )
    };

    SelectedFeatureProviders {
        chat: resolve(ProviderFeature::Chat),
        investigate_issue: resolve(ProviderFeature::InvestigateIssue),
        investigate_pr: resolve(ProviderFeature::InvestigatePr),
        investigate_workflow_run: resolve(ProviderFeature::InvestigateWorkflowRun),
        investigate_security_alert: resolve(ProviderFeature::InvestigateSecurityAlert),
        investigate_advisory: resolve(ProviderFeature::InvestigateAdvisory),
        investigate_linear_issue: resolve(ProviderFeature::InvestigateLinearIssue),
        pr_content_generation: resolve(ProviderFeature::PrContentGeneration),
        commit_message_generation: resolve(ProviderFeature::CommitMessageGeneration),
        code_review: resolve(ProviderFeature::CodeReview),
        context_summarization: resolve(ProviderFeature::ContextSummarization),
        resolve_conflicts: resolve(ProviderFeature::ResolveConflicts),
        release_notes: resolve(ProviderFeature::ReleaseNotes),
        session_naming: resolve(ProviderFeature::SessionNaming),
        branch_naming: resolve(ProviderFeature::BranchNaming),
        session_recap: resolve(ProviderFeature::SessionRecap),
        mcp: resolve(ProviderFeature::Mcp),
        custom_profiles: resolve(ProviderFeature::CustomProfiles),
        usage_reporting: resolve(ProviderFeature::UsageReporting),
        thinking_controls: resolve(ProviderFeature::ThinkingControls),
        chrome_integration: resolve(ProviderFeature::ChromeIntegration),
    }
}

fn build_capabilities(provider: AiProvider) -> AiProviderCapabilities {
    match provider {
        AiProvider::Claude => AiProviderCapabilities {
            chat: true,
            session_naming: true,
            branch_naming: true,
            pr_content_generation: true,
            commit_message_generation: true,
            context_summarization: true,
            session_recap: true,
            code_review: true,
            mcp: true,
            custom_profiles: true,
            usage_reporting: true,
            thinking_controls: true,
            chrome_integration: true,
        },
        AiProvider::Codex => AiProviderCapabilities {
            chat: true,
            session_naming: true,
            branch_naming: true,
            pr_content_generation: true,
            commit_message_generation: true,
            context_summarization: true,
            session_recap: true,
            code_review: true,
            mcp: true,
            custom_profiles: false,
            usage_reporting: true,
            thinking_controls: true,
            chrome_integration: false,
        },
        AiProvider::Opencode => AiProviderCapabilities {
            chat: true,
            session_naming: true,
            branch_naming: true,
            pr_content_generation: true,
            commit_message_generation: true,
            context_summarization: true,
            session_recap: true,
            code_review: true,
            mcp: true,
            custom_profiles: false,
            usage_reporting: false,
            thinking_controls: true,
            chrome_integration: false,
        },
    }
}

fn build_provider_status(
    provider: AiProvider,
    installed: bool,
    authenticated: bool,
    version: Option<String>,
    path: Option<String>,
) -> AiProviderStatus {
    AiProviderStatus {
        installed,
        authenticated,
        available: installed && authenticated,
        version,
        path,
        capabilities: build_capabilities(provider),
    }
}

pub(crate) fn get_provider_status(app: &AppHandle, provider: AiProvider) -> AiProviderStatus {
    match provider {
        AiProvider::Claude => {
            let status = crate::claude_cli::get_claude_cli_status(app).ok();
            let installed = status.as_ref().is_some_and(|value| value.installed);
            let authenticated = if installed {
                crate::claude_cli::get_claude_cli_auth_status(app)
                    .map(|value| value.authenticated)
                    .unwrap_or(false)
            } else {
                false
            };
            build_provider_status(
                provider,
                installed,
                authenticated,
                status.as_ref().and_then(|value| value.version.clone()),
                status.and_then(|value| value.path),
            )
        }
        AiProvider::Codex => {
            let status = crate::codex_cli::get_codex_cli_status(app).ok();
            let installed = status.as_ref().is_some_and(|value| value.installed);
            let authenticated = if installed {
                crate::codex_cli::get_codex_cli_auth_status(app)
                    .map(|value| value.authenticated)
                    .unwrap_or(false)
            } else {
                false
            };
            build_provider_status(
                provider,
                installed,
                authenticated,
                status.as_ref().and_then(|value| value.version.clone()),
                status.and_then(|value| value.path),
            )
        }
        AiProvider::Opencode => {
            let status = crate::opencode_cli::get_opencode_cli_status(app).ok();
            let installed = status.as_ref().is_some_and(|value| value.installed);
            let authenticated = if installed {
                crate::opencode_cli::get_opencode_cli_auth_status(app)
                    .map(|value| value.authenticated)
                    .unwrap_or(false)
            } else {
                false
            };
            build_provider_status(
                provider,
                installed,
                authenticated,
                status.as_ref().and_then(|value| value.version.clone()),
                status.and_then(|value| value.path),
            )
        }
    }
}

pub(crate) fn get_ai_provider_overview_sync(
    app: &AppHandle,
    worktree_id: Option<&str>,
) -> AiProviderOverview {
    let preferences = load_preference_snapshot(app);
    let project_default_backend = load_project_default_backend(app, worktree_id);
    let selected_feature_providers = build_selected_feature_providers(
        &preferences.default_backend,
        project_default_backend.as_deref(),
        &preferences.magic_prompt_backends,
    );

    AiProviderOverview {
        default_chat_provider: selected_feature_providers.chat,
        providers: AiProviderStatuses {
            claude: get_provider_status(app, AiProvider::Claude),
            codex: get_provider_status(app, AiProvider::Codex),
            opencode: get_provider_status(app, AiProvider::Opencode),
        },
        selected_feature_providers,
    }
}

#[tauri::command]
pub async fn get_ai_provider_overview(
    app: AppHandle,
    worktree_id: Option<String>,
) -> Result<AiProviderOverview, String> {
    Ok(get_ai_provider_overview_sync(&app, worktree_id.as_deref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backends_with_session_and_pr(
        session_naming: Option<&str>,
        pr_content: Option<&str>,
    ) -> MagicPromptBackends {
        MagicPromptBackends {
            session_naming_backend: session_naming.map(str::to_string),
            pr_content_backend: pr_content.map(str::to_string),
            ..MagicPromptBackends::default()
        }
    }

    #[test]
    fn capability_matrix_matches_provider_support() {
        let claude = build_capabilities(AiProvider::Claude);
        assert!(claude.custom_profiles);
        assert!(claude.chrome_integration);
        assert!(claude.usage_reporting);

        let codex = build_capabilities(AiProvider::Codex);
        assert!(codex.chat);
        assert!(codex.pr_content_generation);
        assert!(!codex.custom_profiles);
        assert!(!codex.chrome_integration);
        assert!(codex.usage_reporting);

        let opencode = build_capabilities(AiProvider::Opencode);
        assert!(opencode.chat);
        assert!(opencode.code_review);
        assert!(!opencode.custom_profiles);
        assert!(!opencode.usage_reporting);
        assert!(!opencode.chrome_integration);
    }

    #[test]
    fn availability_requires_install_and_authentication() {
        let ready = build_provider_status(
            AiProvider::Codex,
            true,
            true,
            Some("1.2.3".to_string()),
            Some("/tmp/codex".to_string()),
        );
        assert!(ready.available);

        let missing_auth = build_provider_status(AiProvider::Codex, true, false, None, None);
        assert!(!missing_auth.available);

        let missing_install = build_provider_status(AiProvider::Codex, false, false, None, None);
        assert!(!missing_install.available);
    }

    #[test]
    fn resolves_global_project_and_feature_specific_backends() {
        let magic_prompt_backends = backends_with_session_and_pr(Some("codex"), Some("claude"));
        let selected =
            build_selected_feature_providers("claude", Some("opencode"), &magic_prompt_backends);

        assert_eq!(selected.chat, AiProvider::Opencode);
        assert_eq!(selected.pr_content_generation, AiProvider::Claude);
        assert_eq!(selected.session_naming, AiProvider::Codex);
        assert_eq!(selected.branch_naming, AiProvider::Codex);
        assert_eq!(selected.code_review, AiProvider::Opencode);
    }

    #[test]
    fn falls_back_to_global_default_without_project_override() {
        let selected =
            build_selected_feature_providers("codex", None, &MagicPromptBackends::default());

        assert_eq!(selected.chat, AiProvider::Codex);
        assert_eq!(selected.context_summarization, AiProvider::Codex);
        assert_eq!(selected.chrome_integration, AiProvider::Codex);
    }
}
