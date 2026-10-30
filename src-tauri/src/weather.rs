use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub temp: String,
    pub city: String,
    pub desc: String,
    pub icon: String,
}

pub struct WeatherService;

impl WeatherService {
    pub async fn fetch_weather(city: &str) -> Option<WeatherData> {
        let url = if city.trim().is_empty() {
            "https://wttr.in/?format=j1".to_string()
        } else {
            format!("https://wttr.in/{}?format=j1", urlencoding::encode(city.trim()))
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(4))
            .build()
            .ok()?;

        let res = client.get(&url).header("User-Agent", "Mozilla/5.0").send().await.ok()?;
        let json: serde_json::Value = res.json().await.ok()?;

        let current = json.get("current_condition")?.get(0)?;
        let temp_c = current.get("temp_C")?.as_str()?;
        let desc = current.get("weatherDesc")?.get(0)?.get("value")?.as_str()?.to_string();

        let nearest_area = json.get("nearest_area")?.get(0)?;
        let city_name = nearest_area.get("areaName")?.get(0)?.get("value")?.as_str()?.to_string();

        let icon = match desc.to_lowercase().as_str() {
            d if d.contains("sun") || d.contains("clear") => "☀️",
            d if d.contains("cloud") || d.contains("overcast") => "☁️",
            d if d.contains("rain") || d.contains("shower") => "🌧️",
            d if d.contains("snow") => "❄️",
            d if d.contains("thunder") => "⛈️",
            d if d.contains("fog") || d.contains("mist") => "🌫️",
            _ => "⛅",
        };

        Some(WeatherData {
            temp: format!("{}°C", temp_c),
            city: city_name,
            desc,
            icon: icon.to_string(),
        })
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.replace(' ', "+")
    }
}
