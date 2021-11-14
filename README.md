# hass-light-sync

hass-light-sync is a program to capture the display's image and send the average color to a light in Home Assistant. It's similar to solutions like Philips Hue Sync and Ambilight, but only supports a single light at the moment.

# Demo (Video)
[![Demo Video](https://img.youtube.com/vi/iO3uI8IiNZM/0.jpg)](https://youtu.be/iO3uI8IiNZM)

# Installation

## Precompiled Binaries (Windows)
Download the zip-Archive from Releases and edit the `settings.json` file with your server information, and run the `hass-light-sync.exe`.

## Compile it yourself
Install the Rust SDK and clone the git repo. Run:

    cargo build --release 

and copy the `hass-light-sync.exe` from the target folder anywhere you like. Then copy the `settings.json` file from the root of the project to the same directory, and edit it to match your server info.

# Configuration

```json
{
    "api_endpoint": "http://<IP>:<PORT>", 
    "light_entity_name": "light.[entity]",
    "token": "LONG ACCESS TOKEN HERE",  
    "grab_interval": 50,// interval in ms to grab frame
    "skip_pixels": 10,  // only sample n-th pixel (1 -> native, 2 -> 1/2 res etc.)
                        // saves cpu processing

    "smoothing_factor": 0.0, // factor to average out frames:
    // old_frame * smoothing_factor + new_frame * (1-smoothing_factor)   
    // 0 -> disable,   0.25 -> 25 % old frame, 75 % new frame etc.

    "monitor_id": 0 // ID of the target monitor
}
```
