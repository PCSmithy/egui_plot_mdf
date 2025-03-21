use egui::{CollapsingHeader, Ui};
use rsmdf::mdf::{MdfChannel, MDF, MDFFile};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize, PartialEq)]
pub struct SignalTreeState {
    selected_signals: Vec<String>,
    expanded_groups: HashMap<String, bool>,
}

impl SignalTreeState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ui(&mut self, ui: &mut Ui, mdf: Option<&MDF>) {
        ui.heading("Signal Tree");
        
        if let Some(mdf) = mdf {
            let channels = mdf.channels();
            println!("Number of channels: {}", channels.len());  // Debug print
            let groups = self.group_channels(&channels);
            println!("Number of groups: {}", groups.len());  // Debug print
            
            for (group_name, channels) in groups {
                println!("Group '{}' has {} channels", group_name, channels.len());  // Debug print
                CollapsingHeader::new(&group_name)
                    .id_salt(format!("group_{}", group_name))
                    .show(ui, |ui| {
                        for channel in channels {
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