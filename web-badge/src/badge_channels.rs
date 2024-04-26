use std::sync::Mutex;

static UPDATE_FREQ: Mutex<Option<u32>> = Mutex::new(None);
static TEXT: Mutex<Option<String>> = Mutex::new(None);

pub fn set_frequency(freq: u32) {
    UPDATE_FREQ.lock().unwrap().replace(freq);
}

pub fn get_frequency() -> Option<u32> {
    UPDATE_FREQ.lock().unwrap().clone()
}

pub fn set_text(text: impl AsRef<str>) {
    let text = crate::format_text_for_badge(text);
    TEXT.lock().unwrap().replace(text);
}

pub fn get_text() -> Option<String> {
    TEXT.lock().unwrap().clone()
}
