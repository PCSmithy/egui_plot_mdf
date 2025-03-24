use demo::signal_tree::SignalTreeState;
use rsmdf::mdf::{MDF, MDFFile};
use std::path::PathBuf;

fn get_test_mdf_file() -> PathBuf {
    let workspace_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    workspace_dir.join("artifacts/ASAP2_Demo_V171_deflate.mf4")
}

#[test]
fn test_signal_tree_state() {
    let mut state = SignalTreeState::new();
    
    // Test initial state
    assert!(state.get_selected_signals().is_empty());
    
    // Test selecting and deselecting signals
    state.select_channel("test_signal".to_string());
    assert_eq!(state.get_selected_signals().len(), 1);
    assert_eq!(state.get_selected_signals()[0], "test_signal");
    
    state.deselect_channel("test_signal");
    assert!(state.get_selected_signals().is_empty());
}

#[test]
fn test_signal_tree_with_mdf() {
    let test_file = get_test_mdf_file();
    assert!(test_file.exists(), "Test file not found");
    
    let mut mdf = MDF::new(test_file.to_str().unwrap());
    mdf.read_all();
    
    let mut state = SignalTreeState::new();
    let channels = mdf.channels();
    
    // Test that we can group channels
    let groups = state.group_channels(&channels);
    assert!(!groups.is_empty(), "No channel groups found");
    
    // Test selecting a channel from the MDF file
    if let Some(first_channel) = channels.first() {
        state.select_channel(first_channel.name.clone());
        assert!(state.get_selected_signals().contains(&first_channel.name));
        
        state.deselect_channel(&first_channel.name);
        assert!(!state.get_selected_signals().contains(&first_channel.name));
    }
}

#[test]
fn test_signal_tree_search() {
    let test_file = get_test_mdf_file();
    assert!(test_file.exists(), "Test file not found");
    
    let mut mdf = MDF::new(test_file.to_str().unwrap());
    mdf.read_all();
    
    let mut state = SignalTreeState::new();
    let channels = mdf.channels();
    assert!(!channels.is_empty(), "No channels found in test file");

    // Print channel names for debugging
    println!("Available channels:");
    for ch in &channels {
        println!("  - {} ({})", ch.name, ch.full_path());
    }

    // Get the first channel's name to use as a search term
    let first_channel = &channels[0];
    let search_term = &first_channel.name[..std::cmp::min(3, first_channel.name.len())];
    println!("\nUsing search term: '{}'", search_term);
    
    // Test that all channels match when search is empty
    assert!(channels.iter().all(|ch| state.matches_search(ch)), 
        "All channels should match when search is empty");

    // Test partial name search
    state.set_search_query(search_term.to_string());
    let matching = channels.iter()
        .filter(|ch| state.matches_search(ch))
        .count();
    assert!(matching > 0, "No channels matched the search term '{}'", search_term);
    assert!(matching < channels.len(), "All channels matched the search term '{}'", search_term);
    
    // Test case insensitive search
    state.set_search_query(search_term.to_uppercase());
    let matching_upper = channels.iter()
        .filter(|ch| state.matches_search(ch))
        .count();
    assert_eq!(matching, matching_upper, 
        "Case-insensitive search should match the same number of channels");

    // Test regex search
    state.use_regex = true;
    
    // Test simple regex pattern
    state.set_search_query(format!(".*{}.*", search_term));
    let ctx = egui::Context::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            state.ui(ui, Some(&mdf));
        });
    });
    let regex_matching = channels.iter()
        .try_fold(0, |count, ch| -> Result<usize, ()> {
            if state.matches_search(ch) {
                Ok(count + 1)
            } else {
                Ok(count)
            }
        })
        .unwrap();
    println!("\nRegex '{}' matches {} channels", state.search_query, regex_matching);
    assert!(regex_matching > 0, "No channels matched the regex pattern '{}'", state.search_query);
    assert!(regex_matching <= matching, 
        "Regex '.*{}.*' should match fewer or equal channels than substring search", search_term);

    // Test invalid regex pattern
    state.set_search_query("[invalid regex".to_string());
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            state.ui(ui, Some(&mdf));
        });
    });
    let invalid_matches = channels.iter()
        .try_fold(0, |count, ch| -> Result<usize, ()> {
            if state.matches_search(ch) {
                Ok(count + 1)
            } else {
                Ok(count)
            }
        })
        .unwrap();
    assert_eq!(invalid_matches, 0, "Invalid regex should match no channels");

    // Test complex regex pattern (e.g., matching specific path format)
    state.set_search_query(r"\w+/\w+".to_string());  // Matches "group/channel" pattern
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            state.ui(ui, Some(&mdf));
        });
    });
    let path_matches = channels.iter()
        .try_fold(0, |count, ch| -> Result<usize, ()> {
            if state.matches_search(ch) {
                Ok(count + 1)
            } else {
                Ok(count)
            }
        })
        .unwrap();
    assert!(path_matches > 0, "No channels matched the path pattern regex");
}

#[test]
fn test_channel_metadata() {
    let test_file = get_test_mdf_file();
    assert!(test_file.exists(), "Test file not found");
    
    let mut mdf = MDF::new(test_file.to_str().unwrap());
    mdf.read_all();
    
    let state = SignalTreeState::new();
    let channels = mdf.channels();
    assert!(!channels.is_empty(), "No channels found in test file");

    // Test metadata for each channel
    for channel in channels {
        let metadata = state.get_channel_metadata(&channel, &mdf);
        
        // Check basic metadata fields
        assert!(metadata.contains(&channel.name), "Metadata should contain channel name");
        assert!(metadata.contains(&channel.full_path()), "Metadata should contain channel path");
        assert!(metadata.contains("Channel Group:"), "Metadata should contain channel group");
        assert!(metadata.contains("Data: Not yet implemented"), "Metadata should indicate data reading is not implemented");
    }
}

#[test]
fn test_hover_tooltip() {
    let test_file = get_test_mdf_file();
    assert!(test_file.exists(), "Test file not found");
    
    let mut mdf = MDF::new(test_file.to_str().unwrap());
    mdf.read_all();
    
    let mut state = SignalTreeState::new();
    let channels = mdf.channels();
    assert!(!channels.is_empty(), "No channels found in test file");

    // Create a test UI context
    let ctx = egui::Context::default();
    
    // Run the UI and check the response
    ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Get the first channel and its expected metadata
            if let Some(first_channel) = channels.first() {
                let expected_metadata = state.get_channel_metadata(first_channel, &mdf);
                
                // Render the signal tree UI
                state.ui(ui, Some(&mdf));
                
                // Verify the metadata content
                assert!(expected_metadata.contains(&first_channel.name), "Metadata should contain channel name");
                assert!(expected_metadata.contains(&first_channel.full_path()), "Metadata should contain channel path");
                assert!(expected_metadata.contains("Channel Group:"), "Metadata should contain channel group");
                assert!(expected_metadata.contains("Data: Not yet implemented"), "Metadata should indicate data reading is not implemented");
            }
        });
    });
}

#[test]
fn test_regex_search_wipers() {
    // Load the specific MDF file
    let mdf_path = "/Users/smithp/Documents/code/asammdf/src/test_lin_wiper_park_mode[MY_STARFISH[vcfront_297]-wiper_ecu_WIPER_BOSCH].mf4";
    let mut mdf = MDF::new(mdf_path);
    mdf.read_all();
    
    let mut state = SignalTreeState::new();
    let channels = mdf.channels();
    
    // Set up regex search
    state.use_regex = true;
    state.set_search_query("frameState".to_string());
    
    // Get matching channels and their metadata
    let matching_channels: Vec<_> = channels.iter()
        .filter(|ch| state.matches_search(ch))
        .collect();
    
    // Count matching channels
    let matching_count = matching_channels.len();
    
    // Print matching channels and their metadata for debugging
    println!("Matching channels:");
    for channel in matching_channels {
        println!("\nChannel: {} ({})", channel.name, channel.full_path());
        println!("Metadata:");
        let metadata = state.get_channel_metadata(channel, &mdf);
        for line in metadata.lines() {
            println!("  {}", line);
        }
        
        // Check for nested channels
        let parent_path = channel.full_path();
        let nested_channels: Vec<_> = channels.iter()
            .filter(|ch| ch.full_path().starts_with(&parent_path) && ch.full_path() != parent_path)
            .collect();
            
        if !nested_channels.is_empty() {
            println!("  Nested channels:");
            for nested in nested_channels {
                println!("    - {} ({})", nested.name, nested.full_path());
            }
        } else {
            println!("  No nested channels found");
        }
    }
    
    assert_eq!(matching_count, 3, "Expected exactly 3 channels matching the pattern 'LIN_\\w*Wipers'");
} 