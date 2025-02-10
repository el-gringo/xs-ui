/*
pub fn random_byte() -> u8 {
    let num = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("failed to get time since UNIX_EPOCH")
        .subsec_nanos();
    num as u8
}

pub fn apply_mask(buf: &mut [u8], mask: &[u8; 4]) {
    let iter = buf.iter_mut().zip(mask.iter().cycle());
    for (byte, &key) in iter {
        *byte ^= key
    }
}
*/

pub fn mask(masking_key: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    payload
        .iter()
        .zip(masking_key.iter().cycle())
        .map(|(byte, key)| byte ^ key)
        .collect::<Vec<u8>>()
}
