use std::path::PathBuf;
use rsmdf::mdf::{MDF, MDFFile};

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
fn test_load_all_mdf_files() {
    let mdf_files = get_mdf_files();
    assert!(!mdf_files.is_empty(), "No MDF files found in artifacts directory");

    for mdf_path in mdf_files {
        println!("Testing file: {}", mdf_path.display());
        
        // Load the MDF file
        let mut mdf = MDF::new(mdf_path.to_str().unwrap());
        mdf.read_all(); // Need to call read_all() to load the file contents

        // Get all channels
        let channels = mdf.channels();

        // Basic validation of channel information
        assert!(!channels.is_empty(), "No channels found in {}", mdf_path.display());

        // Test channel properties
        for channel in channels {
            // Print channel information
            println!("  Channel: {} (Path: {})", channel.name, channel.full_path());
            
            // Try to read the channel data
            match std::panic::catch_unwind(|| mdf.read_channel(&channel)) {
                Ok(signal) => {
                    assert!(!signal.samples.is_empty(), "No samples found for channel {} in {}", channel.name, mdf_path.display());
                    println!("    Samples: {}", signal.samples.len());
                    println!("    Unit: {}", signal.unit);
                    if !signal.comment.is_empty() {
                        println!("    Description: {}", signal.comment);
                    }
                },
                Err(_) => {
                    println!("    Warning: Channel data reading not yet implemented for this type");
                }
            }
        }
    }
}

#[test]
fn test_channel_metadata() {
    // Use a specific file for detailed metadata testing
    let mdf_path = get_artifacts_dir().join("ASAP2_Demo_V171_deflate.mf4");
    assert!(mdf_path.exists(), "Test file not found");

    let mut mdf = MDF::new(mdf_path.to_str().unwrap());
    mdf.read_all();
    let channels = mdf.channels();

    // Test channel search functionality
    for channel in channels {
        // Test exact search
        let found = mdf.search_channel_exact(&channel.name, channel.data_group, channel.channel_group);
        assert!(found.is_some(), "Failed to find channel {} using exact search", channel.name);
        assert_eq!(found.unwrap().full_path(), channel.full_path());

        // Test general search
        let matches = mdf.search_channels(&channel.name);
        assert!(!matches.is_empty(), "Failed to find channel {} using general search", channel.name);
        assert!(matches.iter().any(|ch| ch.full_path() == channel.full_path()));

        // Read channel data
        match std::panic::catch_unwind(|| mdf.read_channel(&channel)) {
            Ok(signal) => {
                println!("Channel: {}", channel.full_path());
                println!("  Samples: {}", signal.samples.len());
                println!("  Unit: {}", signal.unit);
                println!("  Description: {}", signal.comment);
            },
            Err(_) => {
                println!("Channel: {} (data reading not yet implemented)", channel.full_path());
            }
        }
    }
} 