use anyhow::Result;
use datadog_api_client::datadogV2::api_on_call::{
    CreateOnCallEscalationPolicyOptionalParams, CreateOnCallScheduleOptionalParams,
    GetOnCallEscalationPolicyOptionalParams, GetOnCallScheduleOptionalParams,
    GetUserNotificationRuleOptionalParams, ListUserNotificationRulesOptionalParams, OnCallAPI,
    UpdateOnCallEscalationPolicyOptionalParams, UpdateOnCallScheduleOptionalParams,
    UpdateUserNotificationRuleOptionalParams,
};
use datadog_api_client::datadogV2::api_on_call_paging::OnCallPagingAPI;
use datadog_api_client::datadogV2::api_teams::{
    GetTeamMembershipsOptionalParams, ListTeamsOptionalParams, TeamsAPI,
};
use datadog_api_client::datadogV2::model::{
    CreateOnCallNotificationRuleRequest, CreatePageRequest, CreateUserNotificationChannelRequest,
    EscalationPolicyCreateRequest, EscalationPolicyUpdateRequest, RelationshipToUserTeamUser,
    RelationshipToUserTeamUserData, ScheduleCreateRequest, ScheduleUpdateRequest, TeamCreate,
    TeamCreateAttributes, TeamCreateRequest, TeamType, TeamUpdate, TeamUpdateAttributes,
    TeamUpdateRequest, UpdateOnCallNotificationRuleRequest, UserTeamAttributes, UserTeamCreate,
    UserTeamRelationships, UserTeamRequest, UserTeamRole, UserTeamType, UserTeamUpdate,
    UserTeamUpdateRequest, UserTeamUserType,
};
use regex::Regex;

use crate::config::Config;
use crate::formatter;
use crate::util;

/// Returns true if `s` is a canonical UUID (8-4-4-4-12 lowercase/uppercase hex).
fn is_uuid(s: &str) -> bool {
    // Keep the regex simple and exact; accept upper- or lower-case hex.
    let re = Regex::new(
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$",
    )
    .unwrap();
    re.is_match(s)
}

/// Resolve a team identifier that may be either a UUID or a team handle.
///
/// If `input` parses as a UUID it is returned as-is (fast path, no API call).
/// Otherwise, `ListTeams` is called with `filter[keyword]=<input>` and a single
/// page of size 100. The returned teams are filtered locally for exact
/// `attributes.handle == input` match; exactly one match returns `Ok(id)`.
///
/// Errors out (no silent inference):
///   - no team matches the keyword at all,
///   - substring matches exist but none has an exact handle,
///   - more than one team has an exact handle (defensive; API-side invariant).
///
/// Note: the 100-result ceiling is deliberate; we do not loop-paginate.
/// Handle collisions past page 1 will surface as "no exact match" rather than
/// hiding a real team; callers can still pass the UUID directly.
pub(crate) async fn resolve_team_id(cfg: &Config, input: &str) -> Result<String> {
    if is_uuid(input) {
        return Ok(input.to_string());
    }

    let api = crate::make_api!(TeamsAPI, cfg);
    let params = ListTeamsOptionalParams::default()
        .filter_keyword(input.to_string())
        .page_size(100);
    let resp = api
        .list_teams(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to resolve team handle '{input}': {e:?}"))?;

    let teams = resp.data.unwrap_or_default();
    let total = teams.len();
    let exact: Vec<&datadog_api_client::datadogV2::model::Team> = teams
        .iter()
        .filter(|t| t.attributes.handle == input)
        .collect();

    match exact.len() {
        1 => Ok(exact[0].id.clone()),
        0 if total == 0 => Err(anyhow::anyhow!("no team with handle '{input}'")),
        0 => Err(anyhow::anyhow!(
            "no exact handle match for '{input}' ({total} candidates matched substring)"
        )),
        n => Err(anyhow::anyhow!(
            "no exact handle match for '{input}' ({n} candidates matched substring)"
        )),
    }
}

pub async fn teams_list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(TeamsAPI, cfg);
    let resp = api
        .list_teams(ListTeamsOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list teams: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn teams_get(cfg: &Config, team_id: &str) -> Result<()> {
    let resolved = resolve_team_id(cfg, team_id).await?;
    let api = crate::make_api!(TeamsAPI, cfg);
    let resp = api
        .get_team(resolved)
        .await
        .map_err(|e| anyhow::anyhow!("failed to get team: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn teams_delete(cfg: &Config, team_id: &str) -> Result<()> {
    let resolved = resolve_team_id(cfg, team_id).await?;
    let api = crate::make_api!(TeamsAPI, cfg);
    api.delete_team(resolved.clone())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete team: {e:?}"))?;
    println!("Team '{resolved}' deleted successfully.");
    Ok(())
}

pub async fn teams_create(cfg: &Config, name: &str, handle: &str) -> Result<()> {
    let api = crate::make_api!(TeamsAPI, cfg);
    let attrs = TeamCreateAttributes::new(handle.to_string(), name.to_string());
    let data = TeamCreate::new(attrs, TeamType::TEAM);
    let body = TeamCreateRequest::new(data);
    let resp = api
        .create_team(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create team: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn teams_update(cfg: &Config, team_id: &str, name: &str, handle: &str) -> Result<()> {
    let resolved = resolve_team_id(cfg, team_id).await?;
    let api = crate::make_api!(TeamsAPI, cfg);
    let attrs = TeamUpdateAttributes::new(handle.to_string(), name.to_string());
    let data = TeamUpdate::new(attrs, TeamType::TEAM);
    let body = TeamUpdateRequest::new(data);
    let resp = api
        .update_team(resolved, body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update team: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn memberships_list(cfg: &Config, team_id: &str, page_size: i64) -> Result<()> {
    let resolved = resolve_team_id(cfg, team_id).await?;
    let api = crate::make_api!(TeamsAPI, cfg);
    let params = GetTeamMembershipsOptionalParams::default().page_size(page_size);
    let resp = api
        .get_team_memberships(resolved, params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list memberships: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn memberships_add(
    cfg: &Config,
    team_id: &str,
    user_id: &str,
    role: Option<String>,
) -> Result<()> {
    let resolved = resolve_team_id(cfg, team_id).await?;
    let api = crate::make_api!(TeamsAPI, cfg);
    let mut attrs = UserTeamAttributes::new();
    if let Some(r) = role {
        let team_role = match r.to_lowercase().as_str() {
            "admin" => UserTeamRole::ADMIN,
            _ => UserTeamRole::ADMIN,
        };
        attrs = attrs.role(Some(team_role));
    }
    let user_data =
        RelationshipToUserTeamUserData::new(user_id.to_string(), UserTeamUserType::USERS);
    let user_rel = RelationshipToUserTeamUser::new(user_data);
    let relationships = UserTeamRelationships::new().user(user_rel);
    let data = UserTeamCreate::new(UserTeamType::TEAM_MEMBERSHIPS)
        .attributes(attrs)
        .relationships(relationships);
    let body = UserTeamRequest::new(data);
    let resp = api
        .create_team_membership(resolved, body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to add membership: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn memberships_update(
    cfg: &Config,
    team_id: &str,
    user_id: &str,
    role: &str,
) -> Result<()> {
    let resolved = resolve_team_id(cfg, team_id).await?;
    let api = crate::make_api!(TeamsAPI, cfg);
    let team_role = match role.to_lowercase().as_str() {
        "admin" => UserTeamRole::ADMIN,
        _ => UserTeamRole::ADMIN,
    };
    let attrs = UserTeamAttributes::new().role(Some(team_role));
    let data = UserTeamUpdate::new(UserTeamType::TEAM_MEMBERSHIPS).attributes(attrs);
    let body = UserTeamUpdateRequest::new(data);
    let resp = api
        .update_team_membership(resolved, user_id.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update membership: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn memberships_remove(cfg: &Config, team_id: &str, user_id: &str) -> Result<()> {
    let resolved = resolve_team_id(cfg, team_id).await?;
    let api = crate::make_api!(TeamsAPI, cfg);
    api.delete_team_membership(resolved.clone(), user_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to remove membership: {e:?}"))?;
    println!("Membership for user {user_id} removed from team {resolved}.");
    Ok(())
}

// ---- Escalation Policies ----

pub async fn escalation_policies_get(cfg: &Config, policy_id: &str) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    let resp = api
        .get_on_call_escalation_policy(
            policy_id.to_string(),
            GetOnCallEscalationPolicyOptionalParams::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to get escalation policy: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn escalation_policies_create(cfg: &Config, file: &str) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    let body: EscalationPolicyCreateRequest = util::read_json_file(file)?;
    let resp = api
        .create_on_call_escalation_policy(
            body,
            CreateOnCallEscalationPolicyOptionalParams::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to create escalation policy: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn escalation_policies_update(cfg: &Config, policy_id: &str, file: &str) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    let body: EscalationPolicyUpdateRequest = util::read_json_file(file)?;
    let resp = api
        .update_on_call_escalation_policy(
            policy_id.to_string(),
            body,
            UpdateOnCallEscalationPolicyOptionalParams::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to update escalation policy: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn escalation_policies_delete(cfg: &Config, policy_id: &str) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    api.delete_on_call_escalation_policy(policy_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete escalation policy: {e:?}"))?;
    println!("Escalation policy '{policy_id}' deleted successfully.");
    Ok(())
}

// ---- Schedules ----

pub async fn schedules_get(cfg: &Config, schedule_id: &str) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    let resp = api
        .get_on_call_schedule(
            schedule_id.to_string(),
            GetOnCallScheduleOptionalParams::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to get schedule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn schedules_create(cfg: &Config, file: &str) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    let body: ScheduleCreateRequest = util::read_json_file(file)?;
    let resp = api
        .create_on_call_schedule(body, CreateOnCallScheduleOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to create schedule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn schedules_update(cfg: &Config, schedule_id: &str, file: &str) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    let body: ScheduleUpdateRequest = util::read_json_file(file)?;
    let resp = api
        .update_on_call_schedule(
            schedule_id.to_string(),
            body,
            UpdateOnCallScheduleOptionalParams::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to update schedule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn schedules_delete(cfg: &Config, schedule_id: &str) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    api.delete_on_call_schedule(schedule_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete schedule: {e:?}"))?;
    println!("Schedule '{schedule_id}' deleted successfully.");
    Ok(())
}

// ---- Notification Channels ----

pub async fn notification_channels_list(cfg: &Config, user_id: &str) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    let resp = api
        .list_user_notification_channels(user_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list notification channels: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn notification_channels_get(
    cfg: &Config,
    user_id: &str,
    channel_id: &str,
) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    let resp = api
        .get_user_notification_channel(user_id.to_string(), channel_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get notification channel: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn notification_channels_create(cfg: &Config, user_id: &str, file: &str) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    let body: CreateUserNotificationChannelRequest = util::read_json_file(file)?;
    let resp = api
        .create_user_notification_channel(user_id.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create notification channel: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn notification_channels_delete(
    cfg: &Config,
    user_id: &str,
    channel_id: &str,
) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    api.delete_user_notification_channel(user_id.to_string(), channel_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete notification channel: {e:?}"))?;
    println!("Notification channel '{channel_id}' for user '{user_id}' deleted successfully.");
    Ok(())
}

// ---- Notification Rules ----

pub async fn notification_rules_list(cfg: &Config, user_id: &str) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    let resp = api
        .list_user_notification_rules(
            user_id.to_string(),
            ListUserNotificationRulesOptionalParams::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to list notification rules: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn notification_rules_get(cfg: &Config, user_id: &str, rule_id: &str) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    let resp = api
        .get_user_notification_rule(
            user_id.to_string(),
            rule_id.to_string(),
            GetUserNotificationRuleOptionalParams::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to get notification rule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn notification_rules_create(cfg: &Config, user_id: &str, file: &str) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    let body: CreateOnCallNotificationRuleRequest = util::read_json_file(file)?;
    let resp = api
        .create_user_notification_rule(user_id.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create notification rule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn notification_rules_update(
    cfg: &Config,
    user_id: &str,
    rule_id: &str,
    file: &str,
) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    let body: UpdateOnCallNotificationRuleRequest = util::read_json_file(file)?;
    let resp = api
        .update_user_notification_rule(
            user_id.to_string(),
            rule_id.to_string(),
            body,
            UpdateUserNotificationRuleOptionalParams::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to update notification rule: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn notification_rules_delete(cfg: &Config, user_id: &str, rule_id: &str) -> Result<()> {
    let api = crate::make_api!(OnCallAPI, cfg);
    api.delete_user_notification_rule(user_id.to_string(), rule_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete notification rule: {e:?}"))?;
    println!("Notification rule '{rule_id}' for user '{user_id}' deleted successfully.");
    Ok(())
}

// ---- Pages ----

pub async fn pages_create(cfg: &Config, file: &str) -> Result<()> {
    let api = crate::make_api!(OnCallPagingAPI, cfg);
    let body: CreatePageRequest = util::read_json_file(file)?;
    let resp = api
        .create_on_call_page(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create page: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_uuid_accepts_canonical() {
        assert!(is_uuid("00000000-0000-0000-0000-000000000000"));
        assert!(is_uuid("abcdef01-2345-6789-abcd-ef0123456789"));
        // Uppercase hex is also valid.
        assert!(is_uuid("ABCDEF01-2345-6789-ABCD-EF0123456789"));
    }

    #[test]
    fn test_is_uuid_rejects_handle() {
        assert!(!is_uuid("example-team"));
        assert!(!is_uuid("team-handle-with-dashes"));
        assert!(!is_uuid(""));
    }

    #[test]
    fn test_is_uuid_rejects_wrong_length() {
        // Too short (last segment is 11 hex chars).
        assert!(!is_uuid("00000000-0000-0000-0000-00000000000"));
        // Too long (last segment is 13 hex chars).
        assert!(!is_uuid("00000000-0000-0000-0000-0000000000000"));
        // Non-hex character ('g').
        assert!(!is_uuid("g0000000-0000-0000-0000-000000000000"));
        // Missing dashes.
        assert!(!is_uuid("000000000000000000000000000000000000"));
    }
}
