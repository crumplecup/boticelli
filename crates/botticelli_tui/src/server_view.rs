//! Server management view state and logic.

use botticelli_server::{ModelManager, ModelSpec, ServerConfig, ServerHandle};
use std::path::PathBuf;

/// Server view state.
pub struct ServerView {
    /// Model manager instance
    pub manager: ModelManager,
    /// Current server handle (if running)
    pub server_handle: Option<ServerHandle>,
    /// Available models list
    pub available_models: Vec<ModelInfo>,
    /// Selected model index
    pub selected_model_index: usize,
    /// Download directory
    pub download_dir: PathBuf,
    /// Current operation status
    pub status: String,
    /// Whether a download is in progress
    pub downloading: bool,
}

/// Model information for display.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Model specification
    pub spec: ModelSpec,
    /// Whether the model is downloaded
    pub downloaded: bool,
    /// File path (if downloaded)
    pub path: Option<PathBuf>,
}

impl ServerView {
    /// Create a new server view.
    pub fn new(download_dir: PathBuf) -> Self {
        let manager = ModelManager::new(download_dir.clone());
        
        let available_models = ModelSpec::all()
            .iter()
            .map(|&spec| {
                let downloaded = manager.is_downloaded(spec);
                let path = if downloaded {
                    Some(manager.model_path(spec))
                } else {
                    None
                };
                ModelInfo {
                    spec,
                    downloaded,
                    path,
                }
            })
            .collect();
        
        Self {
            manager,
            server_handle: None,
            available_models,
            selected_model_index: 0,
            download_dir,
            status: "Server stopped".to_string(),
            downloading: false,
        }
    }
    
    /// Select next model in list.
    pub fn select_next(&mut self) {
        if !self.available_models.is_empty() {
            self.selected_model_index = (self.selected_model_index + 1) % self.available_models.len();
        }
    }
    
    /// Select previous model in list.
    pub fn select_previous(&mut self) {
        if !self.available_models.is_empty() {
            if self.selected_model_index == 0 {
                self.selected_model_index = self.available_models.len() - 1;
            } else {
                self.selected_model_index -= 1;
            }
        }
    }
    
    /// Start downloading the selected model.
    pub async fn download_selected(&mut self) -> Result<(), String> {
        if self.downloading {
            return Err("Download already in progress".to_string());
        }
        
        if self.available_models.is_empty() {
            return Err("No models available".to_string());
        }
        
        let model_info = &self.available_models[self.selected_model_index];
        if model_info.downloaded {
            return Err("Model already downloaded".to_string());
        }
        
        let spec = model_info.spec;
        let description = spec.description().to_string();
        
        self.downloading = true;
        self.status = format!("Downloading {}...", description);
        
        match self.manager.download(&spec, "q4").await {
            Ok(path) => {
                self.available_models[self.selected_model_index].downloaded = true;
                self.available_models[self.selected_model_index].path = Some(path.clone());
                self.status = format!("Downloaded {} successfully", description);
                self.downloading = false;
                Ok(())
            }
            Err(e) => {
                self.status = format!("Download failed: {}", e);
                self.downloading = false;
                Err(format!("Download failed: {}", e))
            }
        }
    }
    
    /// Start the server with the selected model.
    pub fn start_server(&mut self) -> Result<(), String> {
        if self.server_handle.is_some() {
            return Err("Server already running".to_string());
        }
        
        if self.available_models.is_empty() {
            return Err("No models available".to_string());
        }
        
        let model_info = &self.available_models[self.selected_model_index];
        if !model_info.downloaded {
            return Err("Model not downloaded. Press 'd' to download.".to_string());
        }
        
        let model_path = model_info.path.as_ref()
            .ok_or("Model path not set".to_string())?
            .clone();
        
        self.status = "Starting server...".to_string();
        
        let config = ServerConfig::new("http://localhost:8080", "default");
        match ServerHandle::start(config, model_path, 8080) {
            Ok(handle) => {
                self.server_handle = Some(handle);
                self.status = format!("Server running on http://localhost:8080 with {}", model_info.spec.description());
                Ok(())
            }
            Err(e) => {
                self.status = format!("Failed to start server: {}", e);
                Err(format!("Failed to start server: {}", e))
            }
        }
    }
    
    /// Stop the server.
    pub fn stop_server(&mut self) -> Result<(), String> {
        if let Some(handle) = self.server_handle.take() {
            self.status = "Stopping server...".to_string();
            match handle.stop() {
                Ok(()) => {
                    self.status = "Server stopped".to_string();
                    Ok(())
                }
                Err(e) => {
                    self.status = format!("Failed to stop server: {}", e);
                    Err(format!("Failed to stop server: {}", e))
                }
            }
        } else {
            Err("Server not running".to_string())
        }
    }
    
    /// Check if server is running.
    pub fn is_running(&self) -> bool {
        self.server_handle.is_some()
    }
}
