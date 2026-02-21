use unicode_normalization::UnicodeNormalization;

/// Builds a normalized compact string used for security checks.
pub fn normalize_for_detection(input: &str) -> String {
    let compatibility_folded = compatibility_jamo_to_syllables(input);
    let compatibility_folded: String = compatibility_folded.nfkc().collect();
    let nfc_normalized: String = compatibility_folded.nfc().collect();
    let lowered = nfc_normalized.to_lowercase();

    lowered.chars().filter(|ch| ch.is_alphanumeric()).collect()
}

fn compatibility_jamo_to_syllables(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut lead: Option<char> = None;
    let mut vowel: Option<char> = None;
    let mut tail: Option<char> = None;

    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if let Some(vowel_jamo) = compat_to_vowel_jamo(ch) {
            match (lead, vowel, tail) {
                (None, None, None) => {
                    lead = Some('ᄋ');
                    vowel = Some(vowel_jamo);
                }
                (Some(_), None, None) => {
                    vowel = Some(vowel_jamo);
                }
                (Some(_), Some(_), None) => {
                    flush_syllable(&mut output, &mut lead, &mut vowel, &mut tail);
                    lead = Some('ᄋ');
                    vowel = Some(vowel_jamo);
                }
                (Some(_), Some(_), Some(tail_jamo)) => {
                    flush_syllable(&mut output, &mut lead, &mut vowel, &mut tail);
                    lead = jong_to_choseong(tail_jamo).or(Some('ᄋ'));
                    vowel = Some(vowel_jamo);
                }
                _ => {
                    flush_syllable(&mut output, &mut lead, &mut vowel, &mut tail);
                    lead = Some('ᄋ');
                    vowel = Some(vowel_jamo);
                }
            }
            continue;
        }

        if let Some(lead_jamo) = compat_to_choseong(ch) {
            match (lead, vowel, tail) {
                (None, None, None) => {
                    lead = Some(lead_jamo);
                }
                (Some(_), None, None) => {
                    flush_syllable(&mut output, &mut lead, &mut vowel, &mut tail);
                    lead = Some(lead_jamo);
                }
                (Some(_), Some(_), None) => {
                    let next_is_vowel = chars
                        .peek()
                        .and_then(|next| compat_to_vowel_jamo(*next))
                        .is_some();

                    if next_is_vowel {
                        flush_syllable(&mut output, &mut lead, &mut vowel, &mut tail);
                        lead = Some(lead_jamo);
                    } else {
                        tail = compat_to_jongseong(ch);
                        if tail.is_none() {
                            flush_syllable(&mut output, &mut lead, &mut vowel, &mut tail);
                            lead = Some(lead_jamo);
                        }
                    }
                }
                (Some(_), Some(_), Some(_)) => {
                    flush_syllable(&mut output, &mut lead, &mut vowel, &mut tail);
                    lead = Some(lead_jamo);
                }
                _ => {
                    flush_syllable(&mut output, &mut lead, &mut vowel, &mut tail);
                    lead = Some(lead_jamo);
                }
            }
            continue;
        }

        flush_syllable(&mut output, &mut lead, &mut vowel, &mut tail);
        output.push(ch);
    }

    flush_syllable(&mut output, &mut lead, &mut vowel, &mut tail);
    output
}

fn flush_syllable(
    out: &mut String,
    lead: &mut Option<char>,
    vowel: &mut Option<char>,
    tail: &mut Option<char>,
) {
    match (*lead, *vowel, *tail) {
        (Some(lc), Some(vc), maybe_tc) => {
            if let Some(syllable) = compose_hangul(lc, vc, maybe_tc) {
                out.push(syllable);
            }
        }
        (Some(lc), None, None) => out.push(lc),
        _ => {}
    }
    *lead = None;
    *vowel = None;
    *tail = None;
}

fn compose_hangul(lead: char, vowel: char, tail: Option<char>) -> Option<char> {
    let lead_index = (lead as u32).checked_sub(0x1100)?;
    let vowel_index = (vowel as u32).checked_sub(0x1161)?;
    if lead_index > 18 || vowel_index > 20 {
        return None;
    }

    let tail_index = match tail {
        Some(value) => {
            let index = (value as u32).checked_sub(0x11A7)?;
            if index > 27 {
                return None;
            }
            index
        }
        None => 0,
    };

    let syllable = 0xAC00 + ((lead_index * 21 + vowel_index) * 28) + tail_index;
    char::from_u32(syllable)
}

fn compat_to_choseong(ch: char) -> Option<char> {
    match ch {
        'ㄱ' => Some('ᄀ'),
        'ㄲ' => Some('ᄁ'),
        'ㄴ' => Some('ᄂ'),
        'ㄷ' => Some('ᄃ'),
        'ㄸ' => Some('ᄄ'),
        'ㄹ' => Some('ᄅ'),
        'ㅁ' => Some('ᄆ'),
        'ㅂ' => Some('ᄇ'),
        'ㅃ' => Some('ᄈ'),
        'ㅅ' => Some('ᄉ'),
        'ㅆ' => Some('ᄊ'),
        'ㅇ' => Some('ᄋ'),
        'ㅈ' => Some('ᄌ'),
        'ㅉ' => Some('ᄍ'),
        'ㅊ' => Some('ᄎ'),
        'ㅋ' => Some('ᄏ'),
        'ㅌ' => Some('ᄐ'),
        'ㅍ' => Some('ᄑ'),
        'ㅎ' => Some('ᄒ'),
        _ => None,
    }
}

fn compat_to_jongseong(ch: char) -> Option<char> {
    match ch {
        'ㄱ' => Some('ᆨ'),
        'ㄲ' => Some('ᆩ'),
        'ㄳ' => Some('ᆪ'),
        'ㄴ' => Some('ᆫ'),
        'ㄵ' => Some('ᆬ'),
        'ㄶ' => Some('ᆭ'),
        'ㄷ' => Some('ᆮ'),
        'ㄹ' => Some('ᆯ'),
        'ㄺ' => Some('ᆰ'),
        'ㄻ' => Some('ᆱ'),
        'ㄼ' => Some('ᆲ'),
        'ㄽ' => Some('ᆳ'),
        'ㄾ' => Some('ᆴ'),
        'ㄿ' => Some('ᆵ'),
        'ㅀ' => Some('ᆶ'),
        'ㅁ' => Some('ᆷ'),
        'ㅂ' => Some('ᆸ'),
        'ㅄ' => Some('ᆹ'),
        'ㅅ' => Some('ᆺ'),
        'ㅆ' => Some('ᆻ'),
        'ㅇ' => Some('ᆼ'),
        'ㅈ' => Some('ᆽ'),
        'ㅊ' => Some('ᆾ'),
        'ㅋ' => Some('ᆿ'),
        'ㅌ' => Some('ᇀ'),
        'ㅍ' => Some('ᇁ'),
        'ㅎ' => Some('ᇂ'),
        _ => None,
    }
}

fn jong_to_choseong(ch: char) -> Option<char> {
    match ch {
        'ᆨ' => Some('ᄀ'),
        'ᆩ' => Some('ᄁ'),
        'ᆫ' => Some('ᄂ'),
        'ᆮ' => Some('ᄃ'),
        'ᆯ' => Some('ᄅ'),
        'ᆷ' => Some('ᄆ'),
        'ᆸ' => Some('ᄇ'),
        'ᆺ' => Some('ᄉ'),
        'ᆻ' => Some('ᄊ'),
        'ᆼ' => Some('ᄋ'),
        'ᆽ' => Some('ᄌ'),
        'ᆾ' => Some('ᄎ'),
        'ᆿ' => Some('ᄏ'),
        'ᇀ' => Some('ᄐ'),
        'ᇁ' => Some('ᄑ'),
        'ᇂ' => Some('ᄒ'),
        _ => None,
    }
}

fn compat_to_vowel_jamo(ch: char) -> Option<char> {
    match ch {
        'ㅏ' => Some('ᅡ'),
        'ㅐ' => Some('ᅢ'),
        'ㅑ' => Some('ᅣ'),
        'ㅒ' => Some('ᅤ'),
        'ㅓ' => Some('ᅥ'),
        'ㅔ' => Some('ᅦ'),
        'ㅕ' => Some('ᅧ'),
        'ㅖ' => Some('ᅨ'),
        'ㅗ' => Some('ᅩ'),
        'ㅘ' => Some('ᅪ'),
        'ㅙ' => Some('ᅫ'),
        'ㅚ' => Some('ᅬ'),
        'ㅛ' => Some('ᅭ'),
        'ㅜ' => Some('ᅮ'),
        'ㅝ' => Some('ᅯ'),
        'ㅞ' => Some('ᅰ'),
        'ㅟ' => Some('ᅱ'),
        'ㅠ' => Some('ᅲ'),
        'ㅡ' => Some('ᅳ'),
        'ㅢ' => Some('ᅴ'),
        'ㅣ' => Some('ᅵ'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_for_detection;

    #[test]
    fn normalizes_korean_nfd_jamo_sequence() {
        let normalized = normalize_for_detection("ㅅㅣㅅㅡㅌㅔㅁ");

        assert_eq!(normalized, "시스템");
    }

    #[test]
    fn strips_korean_whitespace_evasion() {
        let normalized = normalize_for_detection("시 스 템 지 침");

        assert_eq!(normalized, "시스템지침");
    }

    #[test]
    fn strips_english_case_and_punctuation_evasion() {
        let normalized = normalize_for_detection("I.g.n.o.R.e");

        assert_eq!(normalized, "ignore");
    }
}
