use discord_rich_presence::{
    activity::{Activity, Assets, Timestamps},
    DiscordIpc, DiscordIpcClient,
};
use proxy_dll::proxy;
use serde::Deserialize;
use skidscan::signature;
use std::path::Path;

#[derive(Deserialize)]
struct Config {
    #[serde(default)]
    pub show_filename: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            show_filename: true,
        }
    }
}

unsafe fn resolve_static_address(addr: *mut u8) -> *mut u8 {
    let chunk = std::slice::from_raw_parts(addr, 0x100);
    let mut decoder = iced_x86::Decoder::new(64, chunk, iced_x86::DecoderOptions::NONE);
    let mut instr = iced_x86::Instruction::default();

    while decoder.can_decode() {
        decoder.decode_out(&mut instr);
        if instr.is_invalid() {
            continue;
        }

        if instr.op0_kind() == iced_x86::OpKind::Memory
            || instr.op1_kind() == iced_x86::OpKind::Memory
        {
            return instr.memory_displacement64() as *mut u8;
        }
    }

    std::ptr::null_mut()
}

unsafe fn read_filename(sig_result: *mut u8) -> Option<String> {
    let static_addr = sig_result.add(resolve_static_address(sig_result) as usize);
    if static_addr.is_null() {
        return None;
    }

    let sig_ptr = *(static_addr as *mut *mut u8);
    if sig_ptr.is_null() {
        return None;
    }

    let offset_ptr_ptr = sig_ptr.add(8);
    if offset_ptr_ptr.is_null() {
        return None;
    }

    let qstring_ptr = *(offset_ptr_ptr as *mut *mut u8);
    let string_ptr = qstring_ptr.add(24);

    // Read a null terminated UTF16 string - this is jank as fuck
    let size = 0x100;
    let slice = std::slice::from_raw_parts(string_ptr, size * 2);
    let u16_slice: &[u16] = bytemuck::cast_slice(slice);
    let end = u16_slice.iter().position(|c| *c == 0).unwrap_or(0);
    let str = String::from_utf16_lossy(&u16_slice[..end]);

    // if it's a path, just get the filename
    let shortened_str = Path::new(&str)
        .file_name()
        .and_then(|x| x.to_str())
        .map(String::from)
        .unwrap_or(str);

    if shortened_str.trim().is_empty() {
        None
    } else {
        Some(shortened_str)
    }
}

fn set_activity(client: &mut DiscordIpcClient, start: i64, filename: Option<String>) {
    let assets = Assets::new().large_image("010");
    let timestamps = Timestamps::new().start(start);
    let mut activity = Activity::new().assets(assets).timestamps(timestamps);

    // this is dumb as shit i hate lifetimes
    let filename_str = if filename.is_some() {
        format!("Editing {}", filename.as_ref().unwrap())
    } else {
        String::from("")
    };

    if filename.is_some() {
        activity = activity.state(&filename_str);
    }

    client.set_activity(activity).unwrap();
}

fn get_config() -> Config {
    std::env::current_exe()
        .ok()
        .and_then(|p| std::fs::read_to_string(p.join("../discord-rpc.toml")).ok())
        .and_then(|s| toml::from_str::<Config>(&s).ok())
        .unwrap_or_default()
}

#[proxy]
fn main() {
    std::thread::spawn(|| unsafe {
        let config = get_config();

        let filename_ptr = if config.show_filename {
            let filename_sig = signature!("48 89 05 ?? ?? ?? ?? 4D 85 F6");
            filename_sig.scan_module("010Editor.exe").ok()
        } else {
            None
        };

        let mut client = DiscordIpcClient::new("1161367279141933126").unwrap();
        client.connect().unwrap();

        let start = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        set_activity(&mut client, start, None);

        let mut last_filename = None;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));

            // This fucking sucks but I only realized "use an option" now
            // Whatever, commit time :^)
            if let Some(filename_ptr) = filename_ptr {
                let filename = read_filename(filename_ptr);
                if filename != last_filename {
                    last_filename = filename.clone();
                    set_activity(&mut client, start, filename);
                }
            }
        }
    });
}
