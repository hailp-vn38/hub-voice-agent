use super::{InputResampler, is_xvf3800_device_name};

#[test]
fn xvf3800_device_name_matches_case_insensitively() {
    assert!(is_xvf3800_device_name("ReSpeaker XVF3800 USB Audio"));
    assert!(is_xvf3800_device_name("respeaker xvf3800"));
    assert!(!is_xvf3800_device_name("MacBook Pro Speakers"));
}

#[test]
fn resampling_48_khz_input_produces_960_samples_per_60_ms() {
    let input = vec![0; 2_880];
    let mut resampler = InputResampler::new(48_000).unwrap();

    let first_frame = resampler.push(&input);
    let second_frame = resampler.push(&input);

    assert_eq!(first_frame.len(), 960);
    assert_eq!(second_frame.len(), 960);
}

#[test]
fn resampling_keeps_60_ms_output_cadence_across_input_chunks() {
    let input = vec![0; 2_880];
    let mut resampler = InputResampler::new(48_000).unwrap();
    let mut output = Vec::new();

    for chunk in input.chunks(256) {
        output.extend(resampler.push(chunk));
    }

    assert_eq!(output.len(), 960);
}
