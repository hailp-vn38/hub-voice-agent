use super::is_xvf3800_device_name;

#[test]
fn xvf3800_device_name_matches_case_insensitively() {
    assert!(is_xvf3800_device_name("ReSpeaker XVF3800 USB Audio"));
    assert!(is_xvf3800_device_name("respeaker xvf3800"));
    assert!(!is_xvf3800_device_name("MacBook Pro Speakers"));
}
