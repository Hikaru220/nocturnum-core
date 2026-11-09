//! NOCTURNUM: Microsoft sign-in with the Nocturnum Azure application (spec 3.1).
//!
//! Device code flow: login.microsoftonline.com/consumers -> Xbox Live -> XSTS -> login_with_xbox.
//! Replaces the SISU flow with the Minecraft launcher's own client id (we must not use a foreign id).
//! Until Mojang approves the application, `login_with_xbox` answers 403 "Invalid app registration";
//! that is mapped to the error code `minecraft_auth_app_not_approved` and is expected.

use super::minecraft_auth::{MinecraftAuthStep, MinecraftAuthenticationError};
use crate::util::fetch::INSECURE_REQWEST_CLIENT;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};
use std::time::Duration;

/// Public identifier (desktop application, no client secret, no redirect URI).
pub const NOCTURNUM_CLIENT_ID: &str = "b716b6d1-8542-438e-8f05-16a0c7671cd0";
const SCOPE: &str = "XboxLive.signin offline_access";
const DEVICE_CODE_URL: &str =
    "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const TOKEN_URL: &str =
    "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";

type AuthResult<T> = Result<T, MinecraftAuthenticationError>;

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
pub struct DeviceCodeFlow {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Deserialize)]
pub struct MicrosoftTokens {
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: String,
    pub expires_in: u64,
}

fn bad(
    step: MinecraftAuthStep,
    status: StatusCode,
    raw: String,
) -> MinecraftAuthenticationError {
    MinecraftAuthenticationError::DeserializeResponse {
        step,
        raw,
        source: serde::de::Error::custom("unexpected response"),
        status_code: status,
    }
}

fn request_err(
    step: MinecraftAuthStep,
    source: reqwest::Error,
) -> MinecraftAuthenticationError {
    MinecraftAuthenticationError::Request { source, step }
}

async fn read(
    step: MinecraftAuthStep,
    res: reqwest::Response,
) -> AuthResult<(StatusCode, String)> {
    let status = res.status();
    let text = res.text().await.map_err(|e| request_err(step, e))?;
    Ok((status, text))
}

/// Step 1: asks Microsoft for a user code to show to the player.
pub async fn device_code_begin() -> AuthResult<DeviceCodeFlow> {
    let step = MinecraftAuthStep::DeviceCode;
    let res = INSECURE_REQWEST_CLIENT
        .post(DEVICE_CODE_URL)
        .form(&[("client_id", NOCTURNUM_CLIENT_ID), ("scope", SCOPE)])
        .send()
        .await
        .map_err(|e| request_err(step, e))?;
    let (status, text) = read(step, res).await?;
    if !status.is_success() {
        return Err(bad(step, status, text));
    }
    serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            step,
            raw: text,
            source,
            status_code: status,
        }
    })
}

/// Step 2: waits until the player has entered the code (or it has expired).
pub async fn device_code_wait(
    flow: &DeviceCodeFlow,
) -> AuthResult<MicrosoftTokens> {
    let step = MinecraftAuthStep::DeviceCode;
    let mut interval = flow.interval.max(1);
    let deadline = std::time::Instant::now()
        + Duration::from_secs(flow.expires_in.max(60));
    loop {
        tokio::time::sleep(Duration::from_secs(interval)).await;
        if std::time::Instant::now() > deadline {
            return Err(bad(
                step,
                StatusCode::BAD_REQUEST,
                r#"{"error":"expired_token"}"#.to_string(),
            ));
        }
        let res = INSECURE_REQWEST_CLIENT
            .post(TOKEN_URL)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", NOCTURNUM_CLIENT_ID),
                ("device_code", flow.device_code.as_str()),
            ])
            .send()
            .await
            .map_err(|e| request_err(step, e))?;
        let (status, text) = read(step, res).await?;
        if status.is_success() {
            return serde_json::from_str(&text).map_err(|source| {
                MinecraftAuthenticationError::DeserializeResponse {
                    step,
                    raw: text,
                    source,
                    status_code: status,
                }
            });
        }
        let err = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|v| v["error"].as_str().map(String::from))
            .unwrap_or_default();
        match err.as_str() {
            "authorization_pending" => {}
            "slow_down" => interval += 5,
            _ => return Err(bad(step, status, text)),
        }
    }
}

/// Silent refresh: a new Microsoft access token from the stored refresh token.
pub async fn refresh(refresh_token: &str) -> AuthResult<MicrosoftTokens> {
    let step = MinecraftAuthStep::RefreshOAuthToken;
    let res = INSECURE_REQWEST_CLIENT
        .post(TOKEN_URL)
        .form(&[
            ("grant_type", "refresh_token"),
            ("client_id", NOCTURNUM_CLIENT_ID),
            ("refresh_token", refresh_token),
            ("scope", SCOPE),
        ])
        .send()
        .await
        .map_err(|e| request_err(step, e))?;
    let (status, text) = read(step, res).await?;
    if !status.is_success() {
        return Err(bad(step, status, text));
    }
    serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            step,
            raw: text,
            source,
            status_code: status,
        }
    })
}

/// Microsoft token -> Xbox Live -> XSTS -> Minecraft access token.
pub async fn minecraft_access_token(ms_access_token: &str) -> AuthResult<String> {
    // Xbox Live
    let step = MinecraftAuthStep::XboxLive;
    let res = INSECURE_REQWEST_CLIENT
        .post("https://user.auth.xboxlive.com/user/authenticate")
        .header("Accept", "application/json")
        .json(&json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={ms_access_token}"),
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT",
        }))
        .send()
        .await
        .map_err(|e| request_err(step, e))?;
    let (status, text) = read(step, res).await?;
    if !status.is_success() {
        return Err(bad(step, status, text));
    }
    let xbl: Value = serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            step,
            raw: text.clone(),
            source,
            status_code: status,
        }
    })?;
    let xbl_token = xbl["Token"]
        .as_str()
        .ok_or_else(|| bad(step, status, text.clone()))?
        .to_string();

    // XSTS (errors such as 2148916233 / 2148916235 / 2148916238 are mapped by `code()`)
    let step = MinecraftAuthStep::XstsAuthorize;
    let res = INSECURE_REQWEST_CLIENT
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .header("Accept", "application/json")
        .json(&json!({
            "Properties": {"SandboxId": "RETAIL", "UserTokens": [xbl_token]},
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT",
        }))
        .send()
        .await
        .map_err(|e| request_err(step, e))?;
    let (status, text) = read(step, res).await?;
    if !status.is_success() {
        return Err(bad(step, status, text));
    }
    let xsts: Value = serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            step,
            raw: text.clone(),
            source,
            status_code: status,
        }
    })?;
    let xsts_token = xsts["Token"]
        .as_str()
        .ok_or_else(|| bad(step, status, text.clone()))?;
    let uhs = xsts["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .ok_or(MinecraftAuthenticationError::NoUserHash)?;

    // Minecraft Services
    let step = MinecraftAuthStep::MinecraftToken;
    let res = INSECURE_REQWEST_CLIENT
        .post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .header("Accept", "application/json")
        .json(&json!({"identityToken": format!("XBL3.0 x={uhs};{xsts_token}")}))
        .send()
        .await
        .map_err(|e| request_err(step, e))?;
    let (status, text) = read(step, res).await?;
    if !status.is_success() {
        return Err(bad(step, status, text));
    }
    let mc: Value = serde_json::from_str(&text).map_err(|source| {
        MinecraftAuthenticationError::DeserializeResponse {
            step,
            raw: text.clone(),
            source,
            status_code: status,
        }
    })?;
    mc["access_token"]
        .as_str()
        .map(String::from)
        .ok_or_else(|| bad(step, status, text))
}
