#[allow(unused_must_use)]

extern crate captrs;
extern crate reqwest;

use captrs::*;
use std::{time::Duration};
use console::Emoji;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Settings {
    api_endpoint: String,
    light_entity_name: String,
    token: String,
    grab_interval: i16,
    skip_pixels: i16,
    smoothing_factor: f32,
    monitor_id: i16,
}

#[derive(Serialize, Deserialize)]
struct HASSApiBody {
    entity_id: String,
    rgb_color: [u64; 3],
    brightness: u64,
}

#[tokio::main]
async fn main() {
    let term = console::Term::stdout();
    term.set_title("HASS-Light-Sync running...");
    
    println!("{}hass-light-sync - Starting...", Emoji("💡 ", ""));
    println!("{}Reading config...", Emoji("⚙️ ", ""));
    // read settings
    let settingsfile =
        std::fs::read_to_string("settings.json").expect("❌ settings.json file does not exist");


    let settings: Settings =
        serde_json::from_str(settingsfile.as_str()).expect("❌ Failed to parse settings. Please read the configuration section in the README");

    println!("{}Config loaded successfully!", Emoji("✅ ", ""));

    let steps = settings.skip_pixels as u64;
    let grab_interval = settings.grab_interval as u64;
    let smoothing_factor = settings.smoothing_factor;

    // create a capture device
    let mut capturer =
        Capturer::new_with_timeout(settings.monitor_id as usize, Duration::from_millis(1000))
            .expect("❌ Failed to get Capture Object");

    // get the resolution of the monitor
    let (w, h) = capturer.geometry();
    let size = (w as u64 * h as u64) / steps;

    // create http client
    let client = reqwest::Client::new();

    let (mut prev_r, mut prev_g, mut prev_b) = (0, 0, 0);
    
    println!();

    let mut last_timestamp = std::time::Instant::now();

    loop {
        // allocate a vector array for the pixels of the display
        let ps: Vec<Bgr8>;

        // try to grab a frame and fill it into the vector array, if successful, otherwise sleep for 100 ms and skip this frame.
        match capturer.capture_frame() {
            Ok(res) => ps = res,
            Err(error) => {
                println!("{} Failed to grab frame: {:?}", Emoji("❗ ", ""), error);
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
        }

        let (mut total_r, mut total_g, mut total_b) = (0, 0, 0);

        let mut count = 0;

        // for every nth pixel, add the rgb value
        for Bgr8 { r, g, b, .. } in ps.into_iter() {
            if count % steps == 0 {
                total_r += r as u64;
                total_g += g as u64;
                total_b += b as u64;
            }
            count += 1;
        }

        // calculate avg colors
        let (avg_r, avg_g, avg_b) = (total_r / size, total_g / size, total_b / size);

        // smoothing
        let (sm_r, sm_g, sm_b) = (
            smoothing_factor * prev_r as f32 + (1.0 - smoothing_factor) * avg_r as f32,
            smoothing_factor * prev_g as f32 + (1.0 - smoothing_factor) * avg_g as f32,
            smoothing_factor * prev_b as f32 + (1.0 - smoothing_factor) * avg_b as f32,
        );

        // store into prev
        prev_r = sm_r as u64;
        prev_g = sm_g as u64;
        prev_b = sm_b as u64;

        // put into vector
        let avg_arr = vec![prev_r, prev_g, prev_b];

        // get the highest rgb component value -> brightness
        let brightness = avg_arr.iter().max().unwrap();

        let time_elapsed = last_timestamp.elapsed().as_millis();
        last_timestamp = std::time::Instant::now();

        term.move_cursor_up(1);
        term.clear_line();
        println!("{}Current average color: {:?} - Brightness: {} - FPS: {}", Emoji("💡 ", ""), avg_arr, brightness, 1000 / time_elapsed);
        // println!("Avg Color: {:?}    Brightness: {}", avg_arr, brightness);
        send_rgb(&client, &settings, &avg_arr, brightness).await;
        std::thread::sleep(Duration::from_millis(grab_interval));
    }
}



async fn send_rgb(
    client: &reqwest::Client,
    settings: &Settings,
    rgb_vec: &std::vec::Vec<u64>,
    brightness: &u64,
) {
    let api_body = HASSApiBody {
        entity_id: String::from(settings.light_entity_name.as_str()),
        rgb_color: [rgb_vec[0], rgb_vec[1], rgb_vec[2]],
        brightness: *brightness,
    };

    let _response = client
        .post(format!(
            "{}/api/services/light/turn_on",
            settings.api_endpoint.as_str()
        ))
        .header(
            "Authorization",
            format!("Bearer {}", settings.token).as_str(),
        )
        .json(&api_body)
        .send()
        .await;

    match _response{
        Ok(_res) => {
            if _res.status() != 200 {
                println!("{}Connection to Home Assistant failed: HTTP {}", Emoji("❌ ",""), _res.status());
                std::process::exit(0);
            }
        },
        Err(e) => {
            println!("{}Connection to Home Assistant failed: {}", Emoji("❌ ", ""), e);
            std::process::exit(0);
        }
    }
}
