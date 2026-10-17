use std::sync::RwLock;
use std::time::SystemTime;

pub static LOG_ENTRIES: RwLock<Vec<String>> = RwLock::new(Vec::new());

pub fn log_event(msg: impl AsRef<str>) {
    let msg_str = msg.as_ref();
    println!("{}", msg_str);

    let time_str = {
        let now = SystemTime::now();
        if let Ok(duration) = now.duration_since(SystemTime::UNIX_EPOCH) {
            let total_secs = duration.as_secs();
            let secs = total_secs % 60;
            let mins = (total_secs / 60) % 60;
            let hours = ((total_secs / 3600) + 1) % 24;
            format!("{:02}:{:02}:{:02}", hours, mins, secs)
        } else {
            "00:00:00".to_string()
        }
    };

    let formatted = format!("[{}] {}", time_str, msg_str);
    if let Ok(mut lock) = LOG_ENTRIES.write() {
        lock.push(formatted);
        if lock.len() > 100 {
            lock.remove(0);
        }
    }
}
