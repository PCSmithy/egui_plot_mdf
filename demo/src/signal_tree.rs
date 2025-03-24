use egui::{CollapsingHeader, Ui};
use rsmdf::mdf::{MdfChannel, MDF, MDFFile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use regex::Regex;

#[derive(Default, Serialize, Deserialize)]
pub struct SignalTreeState {
    selected_signals: Vec<String>,
    expanded_groups: HashMap<String, bool>,
    #[serde(default)]
    pub search_query: String,
    #[serde(default)]
    pub use_regex: bool,
    #[serde(skip)]
    compiled_regex: Option<Regex>,
    #[serde(skip)]
    regex_error: Option<String>,
}

impl PartialEq for SignalTreeState {
    fn eq(&self, other: &Self) -> bool {
        self.selected_signals == other.selected_signals
            && self.expanded_groups == other.expanded_groups
            && self.search_query == other.search_query
            && self.use_regex == other.use_regex
        // Skip comparing compiled_regex and regex_error
    }
}

impl SignalTreeState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ui(&mut self, ui: &mut Ui, mdf: Option<&MDF>) {
        // Add search bar at the top
        ui.horizontal(|ui| {
            ui.label("Search:");
            let search_changed = ui.text_edit_singleline(&mut self.search_query).changed();
            let regex_changed = ui.checkbox(&mut self.use_regex, "Regex").changed();
            
            if search_changed || regex_changed {
                // Recompile regex if needed
                if self.use_regex && !self.search_query.is_empty() {
                    match Regex::new(&self.search_query) {
                        Ok(re) => {
                            self.compiled_regex = Some(re);
                            self.regex_error = None;
                        }
                        Err(e) => {
                            self.compiled_regex = None;
                            self.regex_error = Some(e.to_string());
                        }
                    }
                } else {
                    self.compiled_regex = None;
                    self.regex_error = None;
                }
            }

            // Show regex error if any
            if let Some(error) = &self.regex_error {
                ui.label(egui::RichText::new(error).color(egui::Color32::RED));
            }
        });
        
        if let Some(mdf) = mdf {
            let channels = mdf.channels();
            let groups = self.group_channels(&channels);
            
            // Sort data group names alphabetically
            let mut data_group_names: Vec<&String> = groups.keys().collect();
            data_group_names.sort();
            
            for data_group_name in data_group_names {
                if let Some(channel_groups) = groups.get(data_group_name) {
                    // Sort channel group names alphabetically
                    let mut channel_group_names: Vec<&String> = channel_groups.keys().collect();
                    channel_group_names.sort();
                    
                    // Create a collapsing header for the data group
                    CollapsingHeader::new(format!("Data Group: {}", data_group_name))
                        .default_open(true)
                        .show(ui, |ui| {
                            // Create nested headers for each channel group
                            for channel_group_name in channel_group_names {
                                if let Some(channels) = channel_groups.get(channel_group_name) {
                                    // Filter channels in this group
                                    let matching_channels: Vec<_> = channels.iter()
                                        .filter(|ch| self.matches_search(ch))
                                        .collect();
                                    
                                    if matching_channels.is_empty() {
                                        continue;
                                    }

                                    // Create a collapsing header for the channel group
                                    CollapsingHeader::new(format!("Channel Group: {}", channel_group_name))
                                        .default_open(true)
                                        .show(ui, |ui| {
                                            for channel in matching_channels {
                                                let mut selected = self.selected_signals.contains(&channel.name);
                                                let response = ui.checkbox(&mut selected, &channel.name);
                                                
                                                if response.changed() {
                                                    if selected {
                                                        self.select_channel(channel.name.clone());
                                                    } else {
                                                        self.deselect_channel(&channel.name);
                                                    }
                                                }
                                            }
                                        });
                                }
                            }
                        });
                }
            }
        } else {
            ui.label("No MDF file loaded");
        }
    }

    pub fn set_search_query(&mut self, query: String) {
        if self.search_query != query {
            self.search_query = query;
            self.compiled_regex = None;
            self.regex_error = None;
        }
    }

    pub fn matches_search(&mut self, channel: &MdfChannel) -> bool {
        if self.search_query.is_empty() {
            return true;
        }

        let search_text = format!("{} {}", channel.name, channel.full_path());

        if self.use_regex {
            // Try to compile regex if not already compiled
            if self.compiled_regex.is_none() && self.regex_error.is_none() {
                match Regex::new(&self.search_query) {
                    Ok(regex) => self.compiled_regex = Some(regex),
                    Err(err) => {
                        self.regex_error = Some(err.to_string());
                        return false;
                    }
                }
            }

            // If we have a regex error, return false
            if self.regex_error.is_some() {
                return false;
            }

            // Use compiled regex if available
            self.compiled_regex.as_ref().map_or(false, |regex| regex.is_match(&search_text))
        } else {
            search_text.to_lowercase().contains(&self.search_query.to_lowercase())
        }
    }

    pub fn group_channels<'a>(&self, channels: &'a [MdfChannel]) -> HashMap<String, HashMap<String, Vec<&'a MdfChannel>>> {
        let mut groups: HashMap<String, HashMap<String, Vec<&'a MdfChannel>>> = HashMap::new();
        
        for channel in channels {
            let full_path = channel.full_path();
            let path_parts: Vec<&str> = full_path.split('/').collect();
            
            // Get data group and channel group
            let data_group = path_parts.get(0).unwrap_or(&"Ungrouped");
            let channel_group = path_parts.get(1).unwrap_or(&"Ungrouped");
            
            // Create nested structure: data_group -> channel_group -> channels
            groups.entry(data_group.to_string())
                .or_default()
                .entry(channel_group.to_string())
                .or_default()
                .push(channel);
        }
        
        // Sort channels within each group by name
        for channel_groups in groups.values_mut() {
            for channels in channel_groups.values_mut() {
                channels.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }
        
        groups
    }

    pub fn select_channel(&mut self, channel_name: String) {
        if !self.selected_signals.contains(&channel_name) {
            self.selected_signals.push(channel_name);
        }
    }

    pub fn deselect_channel(&mut self, channel_name: &str) {
        self.selected_signals.retain(|ch| ch != channel_name);
    }

    pub fn get_selected_signals(&self) -> &[String] {
        &self.selected_signals
    }

    pub fn get_channel_metadata(&self, channel: &MdfChannel, _mdf: &MDF) -> String {
        // Extract channel group from full path
        let full_path = channel.full_path();
        let group = full_path
            .split('/')
            .next()
            .unwrap_or("Ungrouped");
            
        format!("Name: {}\nChannel Group: {}\nPath: {}\nData: Not yet implemented in rsmdf", 
            channel.name, 
            group,
            full_path
        )
    }
} 