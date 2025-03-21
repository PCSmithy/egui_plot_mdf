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