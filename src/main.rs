use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use fedimint_api_client::api::DynGlobalApi;
use fedimint_core::admin_client::SetupStatus;
use fedimint_core::module::ApiAuth;
use fedimint_core::util::SafeUrl;
use serde_json::json;
use tracing::info;
use tracing::debug;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use fedimint_core::endpoint_constants::{
    SETUP_STATUS_ENDPOINT, SET_LOCAL_PARAMS_ENDPOINT,
    ADD_PEER_SETUP_CODE_ENDPOINT, START_DKG_ENDPOINT,
    RESET_PEER_SETUP_CODES_ENDPOINT,
};
use fedimint_core::module::ApiRequestErased;
use fedimint_core::admin_client::SetLocalParamsRequest;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Password for API authentication
    #[arg(short, long)]
    password: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Check the status of the setup process
    Status {
        /// API URL of the guardian (e.g., wss://api.someguardian.com)
        #[arg(short, long)]
        api_url: String,
    },
    /// Set local parameters for this guardian
    SetLocalParams {
        /// API URL of the guardian (e.g., wss://api.someguardian.com)
        #[arg(short, long)]
        api_url: String,
        /// Name of this guardian
        #[arg(short, long)]
        guardian_name: String,
        /// Name of the federation (only needed for the first guardian)
        #[arg(short, long)]
        federation_name: Option<String>,
    },
    /// Add a peer to the federation
    AddPeer {
        /// API URL of the guardian (e.g., wss://api.someguardian.com)
        #[arg(short, long)]
        api_url: String,
        /// Peer connection information (JSON string)
        #[arg(short, long)]
        peer_info: String,
    },
    /// Start the DKG process
    StartDkg {
        /// API URL of the guardian (e.g., wss://api.someguardian.com)
        #[arg(short, long)]
        api_url: String,
    },
    /// Reset the peer setup codes
    ResetPeerSetupCodes {
        /// API URL of the guardian (e.g., wss://api.someguardian.com)
        #[arg(short, long)]
        api_url: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set global default subscriber")?;

    let cli = Cli::parse();

    // Handle commands
    match &cli.command {
        Commands::Status { api_url } => {
            let status = get_setup_status(api_url, &cli.password).await?;
            info!("Setup Status: {status:#?}");
        }
        Commands::SetLocalParams {
            api_url,
            guardian_name,
            federation_name,
        } => {
            let result = set_local_params(
                api_url,
                guardian_name,
                federation_name.clone(),
                &cli.password,
            )
            .await?;
            info!("Set Local Params Result: {result}");
        }
        Commands::AddPeer { api_url, peer_info } => {
            let result = add_peer_connection_info(api_url, peer_info, &cli.password).await?;
            info!("Add Peer Result: {result}");
        }
        Commands::StartDkg { api_url } => {
            start_dkg(api_url, &cli.password).await?;
            info!("DKG process started successfully");
        }
        Commands::ResetPeerSetupCodes { api_url } => {
            reset_peer_setup_codes(api_url, &cli.password).await?;
            info!("Setup reset successfully");
        }
    }

    Ok(())
}

// Helper to log the JSON-RPC request as a curl command
fn log_curl_request(api_url: &str, method: &str, request: &ApiRequestErased) {
    let rpc = json!({
        "jsonrpc": "2.0",
        "id": 0,
        "method": method,
        "params": [request.to_json()],
    });
    let base = api_url.trim_end_matches('/');
    let curl_url = base.replacen("ws://", "http://", 1);
    debug!("CURL: curl -X POST \"{}\" -H \"Content-Type: application/json\" -d '{}' -k", curl_url, rpc.to_string());
}

/// Get the current setup status
async fn get_setup_status(api_url: &str, password: &str) -> Result<SetupStatus> {
    let safe_url = SafeUrl::parse(api_url).context("Failed to parse API URL")?;
    let api_secret = Some(password.to_string());

    // Create the DynGlobalApi client
    let api = DynGlobalApi::from_setup_endpoint(safe_url, &api_secret)
        .await
        .context("Failed to create API client")?;

    // Prepare and log curl
    let auth = ApiAuth(password.to_string());
    let request = ApiRequestErased::default().with_auth(auth.clone());
    log_curl_request(api_url, SETUP_STATUS_ENDPOINT, &request);

    // Get setup status
    let status = api
        .setup_status(auth)
        .await
        .context("Failed to get setup status")?;

    Ok(status)
}

/// Set local parameters for a guardian
async fn set_local_params(
    api_url: &str,
    guardian_name: &str,
    federation_name: Option<String>,
    password: &str,
) -> Result<String> {
    let safe_url = SafeUrl::parse(api_url).context("Failed to parse API URL")?;
    let api_secret = Some(password.to_string());

    // Create the DynGlobalApi client
    let api = DynGlobalApi::from_setup_endpoint(safe_url, &api_secret)
        .await
        .context("Failed to create API client")?;

    // Prepare and log curl
    let auth = ApiAuth(password.to_string());
    let params = SetLocalParamsRequest { name: guardian_name.to_string(), federation_name: federation_name.clone() };
    let request = ApiRequestErased::new(params).with_auth(auth.clone());
    log_curl_request(api_url, SET_LOCAL_PARAMS_ENDPOINT, &request);

    // Set local parameters
    let result = api
        .set_local_params(guardian_name.to_string(), federation_name, auth)
        .await
        .context("Failed to set local parameters")?;

    Ok(result)
}

/// Add peer connection information
async fn add_peer_connection_info(
    api_url: &str,
    peer_info: &str,
    password: &str,
) -> Result<String> {
    let safe_url = SafeUrl::parse(api_url).context("Failed to parse API URL")?;
    let api_secret = Some(password.to_string());

    // Create the DynGlobalApi client
    let api = DynGlobalApi::from_setup_endpoint(safe_url, &api_secret)
        .await
        .context("Failed to create API client")?;

    // Prepare and log curl
    let auth = ApiAuth(password.to_string());
    let request = ApiRequestErased::new(peer_info.to_string()).with_auth(auth.clone());
    log_curl_request(api_url, ADD_PEER_SETUP_CODE_ENDPOINT, &request);

    // Add peer connection info
    let result = api
        .add_peer_connection_info(peer_info.to_string(), auth)
        .await
        .context("Failed to add peer connection info")?;

    Ok(result)
}

/// Start the DKG process
async fn start_dkg(api_url: &str, password: &str) -> Result<()> {
    let safe_url = SafeUrl::parse(api_url).context("Failed to parse API URL")?;
    let api_secret = Some(password.to_string());

    // Create the DynGlobalApi client
    let api = DynGlobalApi::from_setup_endpoint(safe_url, &api_secret)
        .await
        .context("Failed to create API client")?;

    // Prepare and log curl
    let auth = ApiAuth(password.to_string());
    let request = ApiRequestErased::default().with_auth(auth.clone());
    log_curl_request(api_url, START_DKG_ENDPOINT, &request);

    // Start DKG
    api.start_dkg(auth).await.context("Failed to start DKG")?;

    Ok(())
}

/// Reset the peer setup codes
async fn reset_peer_setup_codes(api_url: &str, password: &str) -> Result<()> {
    let safe_url = SafeUrl::parse(api_url).context("Failed to parse API URL")?;
    let api_secret = Some(password.to_string());

    // Create the DynGlobalApi client
    let api = DynGlobalApi::from_setup_endpoint(safe_url, &api_secret)
        .await
        .context("Failed to create API client")?;

    // Prepare and log curl
    let auth = ApiAuth(password.to_string());
    let request = ApiRequestErased::default().with_auth(auth.clone());
    log_curl_request(api_url, RESET_PEER_SETUP_CODES_ENDPOINT, &request);

    api.reset_peer_setup_codes(auth)
        .await
        .context("Failed to reset peer setup codes")?;

    Ok(())
}
