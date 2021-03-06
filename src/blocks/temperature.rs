use std::time::Duration;
use std::process::Command;
use std::error::Error;

use block::Block;
use widgets::button::ButtonWidget;
use widget::{I3BarWidget, State};
use input::I3barEvent;

use serde_json::Value;
use uuid::Uuid;


pub struct Temperature {
    text: ButtonWidget,
    output: String,
    collapsed: bool,
    id: String,
    update_interval: Duration,
}

impl Temperature {
    pub fn new(config: Value, theme: Value) -> Temperature {
        {
            let id = Uuid::new_v4().simple().to_string();
            Temperature {
                update_interval: Duration::new(get_u64_default!(config, "interval", 5), 0),
                text: ButtonWidget::new(theme.clone(), &id).with_icon("thermometer"),
                output: String::new(),
                collapsed: get_bool_default!(config, "collapsed", true),
                id,
            }
        }
        
    }
}


impl Block for Temperature
{
    fn update(&mut self) -> Option<Duration> {
        let output = Command::new("sensors")
            .args(&["-u"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
            .unwrap_or_else(|e| e.description().to_owned());

        let mut temperatures: Vec<u64> = Vec::new();

        for line in output.lines() {
            if line.starts_with("  temp") {
                let rest = &line[6..]
                    .split('_')
                    .flat_map(|x| x.split(' '))
                    .flat_map(|x| x.split('.'))
                    .collect::<Vec<_>>();

                if rest[1].starts_with("input") {
                    temperatures.push(rest[2].parse::<u64>().unwrap());
                }
            }
        }

        if !temperatures.is_empty() {
            let max: u64 = *temperatures.iter().max().unwrap();
            let avg: u64 = (temperatures.iter().sum::<u64>() as f64 /
                temperatures.len() as f64).round() as u64;

            self.output = format!("{}° avg, {}° max", avg, max);
            if !self.collapsed {
                self.text.set_text(self.output.clone());
            }

            self.text.set_state(match max {
                0 ... 20 => State::Good,
                20 ... 45 => State::Idle,
                45 ... 60 => State::Info,
                60 ... 80 => State::Warning,
                _ => State::Critical
            })
        }

        Some(self.update_interval.clone())
    }
    fn view(&self) -> Vec<&I3BarWidget> {
        vec![&self.text]
    }
    fn click(&mut self, e: &I3barEvent) {
        if let Some(ref name) = e.name {
            if name.as_str() == self.id {
                self.collapsed = !self.collapsed;
                if self.collapsed {
                    self.text.set_text(String::new());
                } else {
                    self.text.set_text(self.output.clone());
                }
            }
        }
    }
    fn id(&self) -> &str {
        &self.id
    }
}