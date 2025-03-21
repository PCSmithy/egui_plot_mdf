use std::path::PathBuf;
use rsmdf::mdf::{MDF, MDFFile};
use std::env;

pub struct MdfState {
    file_path: Option<PathBuf>,
    mdf: Option<MDF>,
    selected_channels: Vec<String>,
}

impl MdfState {
    pub fn new() -> Self {
        Self {
            file_path: None,
            mdf: None,
            selected_channels: Vec::new(),
        }
    }

    pub fn load_file(&mut self, path: PathBuf) -> Result<(), String> {
        // Change to the workspace root directory
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let workspace_dir = manifest_dir
            .parent()
            .ok_or("Failed to get workspace directory")?;
        env::set_current_dir(&workspace_dir).map_err(|e| e.to_string())?;
        
        // Convert absolute path to relative path from workspace root
        let relative_path = path.strip_prefix(&workspace_dir)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| path.clone());
        
        println!("Current dir when loading file: {:?}", env::current_dir().unwrap());
        println!("Loading file: {:?}", relative_path);
        
        // Verify the file exists before trying to read it
        if !relative_path.exists() {
            return Err(format!("File not found: {:?}", relative_path));
        }
        
        let path_str = relative_path.to_str().ok_or("Invalid path")?;
        
        // Use std::panic::catch_unwind to handle potential panics from MDF::new
        let mdf = std::panic::catch_unwind(|| {
            let mut mdf = MDF::new(path_str);
            mdf.read_all();
            mdf
        }).map_err(|_| format!("Failed to read MDF file: {:?}", relative_path))?;
        
        self.file_path = Some(path);
        self.mdf = Some(mdf);
        self.selected_channels.clear();
        Ok(())
    }

    pub fn get_channel_names(&self) -> Vec<String> {
        if let Some(mdf) = &self.mdf {
            mdf.channels()
                .iter()
                .map(|ch| ch.name.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn select_channel(&mut self, channel_name: String) {
        if !self.selected_channels.contains(&channel_name) {
            self.selected_channels.push(channel_name);
        }
    }

    pub fn deselect_channel(&mut self, channel_name: &str) {
        self.selected_channels.retain(|ch| ch != channel_name);
    }

    pub fn get_selected_channels(&self) -> &[String] {
        &self.selected_channels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper function to get the path to the artifacts directory
    fn get_artifacts_dir() -> PathBuf {
        let workspace_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf();
        workspace_dir.join("artifacts")
    }

    /// Helper function to get all MDF files in the artifacts directory
    fn get_mdf_files() -> Vec<PathBuf> {
        let artifacts_dir = get_artifacts_dir();
        std::fs::read_dir(artifacts_dir)
            .unwrap()
            .filter_map(|entry| {
                let entry = entry.unwrap();
                let path = entry.path();
                // For now, only test with the demo files that we know work
                if path.file_name().map_or(false, |name| {
                    name.to_string_lossy().contains("ASAP2_Demo") || name.to_string_lossy().contains("Discrete")
                }) {
                    Some(path)
                } else {
                    None
                }
            })
            .collect()
    }

    #[test]
    fn test_mdf_state_initialization() {
        let state = MdfState::new();
        assert!(state.file_path.is_none());
        assert!(state.mdf.is_none());
        assert!(state.selected_channels.is_empty());
    }

    #[test]
    fn test_mdf_file_loading() {
        let mut state = MdfState::new();
        let mdf_files = get_mdf_files();
        println!("Found MDF files: {:?}", mdf_files);
        assert!(!mdf_files.is_empty(), "No MDF files found in artifacts directory");
        let test_file = &mdf_files[0];
        println!("Using test file: {:?}", test_file);
        println!("Test file exists: {}", test_file.exists());
        
        // Test loading a valid file
        let result = state.load_file(test_file.clone());
        assert!(result.is_ok());
        assert!(state.file_path.is_some());
        assert!(state.mdf.is_some());
        
        // Verify channels are available
        let channels = state.get_channel_names();
        assert!(!channels.is_empty());
        
        // Test loading a non-existent file
        let result = state.load_file(PathBuf::from("nonexistent.mf4"));
        assert!(result.is_err());
    }

    #[test]
    fn test_channel_selection() {
        let mut state = MdfState::new();
        let mdf_files = get_mdf_files();
        assert!(!mdf_files.is_empty(), "No MDF files found in artifacts directory");
        let test_file = &mdf_files[0];
        state.load_file(test_file.clone()).unwrap();
        
        let channels = state.get_channel_names();
        assert!(!channels.is_empty());
        
        // Test selecting a channel
        let first_channel = channels[0].clone();
        state.select_channel(first_channel.clone());
        assert_eq!(state.get_selected_channels().len(), 1);
        assert_eq!(state.get_selected_channels()[0], first_channel);
        
        // Test selecting the same channel again (should not duplicate)
        state.select_channel(first_channel.clone());
        assert_eq!(state.get_selected_channels().len(), 1);
        
        // Test deselecting a channel
        state.deselect_channel(&first_channel);
        assert!(state.get_selected_channels().is_empty());
    }
} 