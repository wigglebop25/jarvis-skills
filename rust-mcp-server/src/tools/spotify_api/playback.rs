use std::time::Duration;

use reqwest::StatusCode;
use serde_json::json;
use urlencoding::encode;

use super::SpotifyClient;

impl SpotifyClient {
    pub async fn play_on_best_device(&self, uri: &str) -> Result<(), String> {
        self.play_with_uri(uri, "Failed to play on device: ", "Failed to play: ")
            .await
    }

    pub async fn play_playback(&self, uri: &str) -> Result<(), String> {
        self.play_with_uri(uri, "Failed to play: ", "Failed to play: ")
            .await
    }

    pub async fn pause_playback(&self) -> Result<(), String> {
        let token = self.get_access_token().await?;

        let response = self
            .http_client
            .put("https://api.spotify.com/v1/me/player/pause")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Length", "0")
            .body("")
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to pause: {}", e))?;

        playback_status_result(response.status(), "Failed to pause")
    }

    pub async fn next_track(&self) -> Result<(), String> {
        let token = self.get_access_token().await?;

        let response = self
            .http_client
            .post("https://api.spotify.com/v1/me/player/next")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Length", "0")
            .body("")
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to skip to next: {}", e))?;

        playback_status_result(response.status(), "Failed to skip to next")
    }

    pub async fn previous_track(&self) -> Result<(), String> {
        let token = self.get_access_token().await?;

        let response = self
            .http_client
            .post("https://api.spotify.com/v1/me/player/previous")
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Length", "0")
            .body("")
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to skip to previous: {}", e))?;

        playback_status_result(response.status(), "Failed to skip to previous")
    }

    pub async fn set_volume_for_device(
        &self,
        volume_percent: u8,
        device_id: Option<&str>,
    ) -> Result<(), String> {
        let token = self.get_access_token().await?;
        let volume = volume_percent.min(100);
        let mut url = format!(
            "https://api.spotify.com/v1/me/player/volume?volume_percent={}",
            volume
        );
        if let Some(device_id) = device_id {
            url.push_str("&device_id=");
            url.push_str(&encode(device_id));
        }

        let response = self
            .http_client
            .put(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Length", "0")
            .body("")
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to set volume: {}", e))?;

        playback_status_result(response.status(), "Failed to set volume")
    }

    pub async fn resume_playback(&self, device_id: Option<&str>) -> Result<(), String> {
        let token = self.get_access_token().await?;
        let mut url = "https://api.spotify.com/v1/me/player/play".to_string();
        if let Some(device_id) = device_id {
            url.push_str("?device_id=");
            url.push_str(&encode(device_id));
        }

        let response = self
            .http_client
            .put(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Length", "0")
            .body("")
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to resume playback: {}", e))?;

        playback_status_result(response.status(), "Failed to resume playback")
    }

    pub async fn add_to_queue(&self, uri: &str, device_id: Option<&str>) -> Result<(), String> {
        let token = self.get_access_token().await?;
        let mut url = format!(
            "https://api.spotify.com/v1/me/player/queue?uri={}",
            encode(uri)
        );
        if let Some(device_id) = device_id {
            url.push_str("&device_id=");
            url.push_str(&encode(device_id));
        }

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Length", "0")
            .body("")
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to add to queue: {}", e))?;

        playback_status_result(response.status(), "Failed to add to queue")
    }

    pub async fn get_volume_for_device(&self, device_id: Option<&str>) -> Result<u8, String> {
        let devices = self.get_devices().await?;
        let selected_device = if let Some(device_id) = device_id {
            devices
                .iter()
                .find(|device| device.id == device_id)
                .ok_or_else(|| format!("Device not found: {}", device_id))?
        } else {
            devices
                .iter()
                .find(|device| device.is_active)
                .ok_or_else(|| "No active Spotify device found".to_string())?
        };

        selected_device
            .volume_percent
            .ok_or_else(|| format!("Volume unavailable for device: {}", selected_device.name))
    }

    async fn play_with_uri(
        &self,
        uri: &str,
        request_error_prefix: &str,
        status_error_prefix: &str,
    ) -> Result<(), String> {
        let token = self.get_access_token().await?;

        let body = json!({
            "uris": [uri],
            "position_ms": 0
        });

        let response = self
            .http_client
            .put("https://api.spotify.com/v1/me/player/play")
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("{request_error_prefix}{e}"))?;

        playback_status_result(response.status(), status_error_prefix)
    }
}

fn playback_status_result(status: StatusCode, error_prefix: &str) -> Result<(), String> {
    if status.is_success() || status.as_u16() == 204 {
        Ok(())
    } else {
        Err(format!("{error_prefix}{status}"))
    }
}
