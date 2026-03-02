use xs_layout::TextProperties;

pub enum Message {
    /// src url
    LoadImage(String),
    /// text, text-properties
    Text(String, TextProperties)
}
