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
        ui.heading("Signal Tree");
        
        // Add search bar with regex toggle
        ui.horizontal(|ui| {
            ui.label("🔍");
            let search_changed = ui.text_edit_singleline(&mut self.search_query).changed();
            let regex_changed = ui.checkbox(&mut self.use_regex, "Regex").changed();
            
            if search_changed || regex_changed {
                // Clear expanded states when search changes
                self.expanded_groups.clear();
                
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
            
            for (group_name, channels) in groups {
                // Skip groups with no matching channels
                let matching_channels: Vec<_> = channels.into_iter()
                    .filter(|ch| self.matches_search(ch))
                    .collect();
                
                if matching_channels.is_empty() {
                    continue;
                }

                // Auto-expand groups when searching
                let is_searching = !self.search_query.is_empty();
                CollapsingHeader::new(&group_name)
                    .id_salt(format!("group_{}", group_name))
                    .default_open(is_searching)
                    .show(ui, |ui| {
                        for channel in matching_channels {
                            let mut selected = self.selected_signals.contains(&channel.name);
                            if ui.checkbox(&mut selected, &channel.name).changed() {
                                if selected {
                                    self.select_channel(channel.name.clone());
                                } else {
                                    self.deselect_channel(&channel.name);
                                }
                            }
                            
                            // Show channel metadata if available
                            ui.small(format!("Path: {}", channel.full_path()));
                        }
                    });
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

    pub fn group_channels<'a>(&self, channels: &'a [MdfChannel]) -> HashMap<String, Vec<&'a MdfChannel>> {
        let mut groups: HashMap<String, Vec<&MdfChannel>> = HashMap::new();
        
        for channel in channels {
            let group_name = channel.full_path()
                .split('/')
                .next()
                .unwrap_or("Ungrouped")
                .to_string();
            
            groups.entry(group_name)
                .or_default()
                .push(channel);
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
} 