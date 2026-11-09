//! 입력 규칙(docs/api.md §2)과 단어 목록(`backend/data/`). 금지어·욕설 검사는 채팅·방송·후원도 쓴다.

const BANNED_WORDS: &str = include_str!("../../data/banned_words.txt");
const PROFANITY: &str = include_str!("../../data/profanity_ko.txt");
const COMMON_PASSWORDS: &str = include_str!("../../data/common_passwords.txt");

/// 닉네임에 들어 있으면 안 되는 말(대소문자 무시 부분 일치). 띄어쓰기 없는 형태로 적는다.
const RESERVED_NICKNAMES: [&str; 10] = [
    "운영자",
    "관리자",
    "운영팀",
    "flutch",
    "플러치",
    "admin",
    "system",
    "시스템",
    "탈퇴한사용자",
    "익명의후원자",
];

fn entries(list: &str) -> impl Iterator<Item = &str> {
    list.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
}

fn contains_any(list: &str, text: &str) -> bool {
    let text = text.to_lowercase();
    entries(list).any(|w| text.contains(&w.to_lowercase()))
}

/// FLUTCH 금지어가 들어 있으면 true → 전송·저장을 막는다(채팅·닉네임·방송 제목·태그·후원 메시지).
pub fn contains_banned_word(text: &str) -> bool {
    contains_any(BANNED_WORDS, text)
}

/// 일반 욕설이 들어 있으면 true. 닉네임·방송 제목은 거절, 채팅·후원 메시지는 가림 처리에 쓴다.
pub fn contains_profanity(text: &str) -> bool {
    contains_any(PROFANITY, text)
}

/// `login_id`는 이미 소문자로 바꾼 값.
pub fn check_login_id(login_id: &str) -> Result<(), &'static str> {
    let len = login_id.chars().count();
    if (4..=20).contains(&len)
        && login_id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    {
        Ok(())
    } else {
        Err("영문 소문자·숫자 4~20자로 입력해 주세요.")
    }
}

pub fn check_password(password: &str, login_id: &str) -> Result<(), &'static str> {
    let len = password.chars().count();
    if !(8..=64).contains(&len) {
        return Err("8~64자로 입력해 주세요.");
    }
    if !password.chars().any(|c| c.is_ascii_alphabetic())
        || !password.chars().any(|c| c.is_ascii_digit())
    {
        return Err("영문과 숫자를 함께 넣어 주세요.");
    }
    let lower = password.to_lowercase();
    if lower == login_id {
        return Err("아이디와 다른 비밀번호를 써 주세요.");
    }
    if entries(COMMON_PASSWORDS).any(|p| p.to_lowercase() == lower) {
        return Err("너무 흔한 비밀번호예요. 다른 비밀번호를 써 주세요.");
    }
    Ok(())
}

/// `nickname`은 앞뒤 공백을 자른 값.
pub fn check_nickname(nickname: &str) -> Result<(), &'static str> {
    let len = nickname.chars().count();
    if !(2..=12).contains(&len) {
        return Err("2~12자로 입력해 주세요.");
    }
    if !nickname
        .chars()
        .all(|c| ('가'..='힣').contains(&c) || c.is_ascii_alphanumeric())
    {
        return Err("한글·영문·숫자만 쓸 수 있어요.");
    }
    let lower = nickname.to_lowercase();
    if RESERVED_NICKNAMES.iter().any(|r| lower.contains(r))
        || contains_profanity(nickname)
        || contains_banned_word(nickname)
    {
        return Err("사용할 수 없는 닉네임이에요.");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_id_rules() {
        assert!(check_login_id("abc1").is_ok());
        assert!(check_login_id("abc").is_err());
        assert!(check_login_id("abc_1").is_err());
        assert!(check_login_id(&"a".repeat(21)).is_err());
    }

    #[test]
    fn password_rules() {
        assert!(check_password("goodpass1", "user1").is_ok());
        assert!(check_password("short1", "user1").is_err());
        assert!(check_password("onlyletters", "user1").is_err());
        assert!(check_password("12345678", "user1").is_err());
        assert!(check_password("User1234", "user1234").is_err());
        assert!(check_password("1QAZ2WSX", "user1").is_err()); // 흔한 비밀번호(대소문자 무시)
    }

    #[test]
    fn nickname_rules() {
        assert!(check_nickname("플레이어1").is_ok());
        assert!(check_nickname("가").is_err());
        assert!(check_nickname("공백 있음").is_err());
        assert!(check_nickname("ㅋㅋ").is_err()); // 자모는 글자 아님
        assert!(check_nickname("진짜Admin").is_err());
        assert!(check_nickname("FLUTCH팬").is_err());
        assert!(check_nickname("씨발맨").is_err());
        assert!(check_nickname("토토사이트").is_err());
    }

    #[test]
    fn word_checks_ignore_case_and_skip_comments() {
        assert!(contains_profanity("what the FUCK"));
        assert!(!contains_profanity("유모차 끌고 산책")); // 오탐 항목은 목록에서 뺐다
        assert!(contains_banned_word("바카라 하실 분"));
        assert!(!contains_banned_word("# 불법 도박·약물 홍보")); // 주석 줄은 단어가 아님
    }
}
