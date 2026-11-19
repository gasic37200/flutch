import type { Donation } from '../../api/generated/Donation.ts'

const MAX_MESSAGE = 100
const KEY = 'flutch.donationTts'
const MAX_JAMO_RUN = 8

// 호환 자모(ㄱ U+3131 ~ ㅣ U+3163)를 읽을 수 있는 음절로. speechSynthesis는 "ㅋㅋㅋ"·"ㅠㅠ"를 건너뛰거나 글자 이름으로 읽는다.
const JAMO: Record<string, string> = {
  ㄱ: '그', ㄲ: '끄', ㄳ: '극스', ㄴ: '느', ㄵ: '는즈', ㄶ: '는흐', ㄷ: '드', ㄸ: '뜨', ㄹ: '르', ㄺ: '륵', ㄻ: '름', ㄼ: '릅', ㄽ: '릇', ㄾ: '르트', ㄿ: '르프', ㅀ: '르흐',
  ㅁ: '므', ㅂ: '브', ㅃ: '쁘', ㅄ: '븝스', ㅅ: '스', ㅆ: '쓰', ㅇ: '응', ㅈ: '즈', ㅉ: '쯔', ㅊ: '츠', ㅋ: '크', ㅌ: '트', ㅍ: '프', ㅎ: '흐',
  ㅏ: '아', ㅐ: '애', ㅑ: '야', ㅒ: '얘', ㅓ: '어', ㅔ: '에', ㅕ: '여', ㅖ: '예', ㅗ: '오', ㅘ: '와', ㅙ: '왜', ㅚ: '외', ㅛ: '요', ㅜ: '우', ㅝ: '워', ㅞ: '웨', ㅟ: '위', ㅠ: '유', ㅡ: '으', ㅢ: '의', ㅣ: '이',
}

/** 같은 자모 연속은 최대 8개로 줄이고, 자모를 읽을 수 있는 음절로 바꾼다. */
export const readableJamo = (s: string) =>
  s.replace(/([ㄱ-ㅣ])\1*/g, (run, j: string) => (JAMO[j] ?? j).repeat(Math.min(run.length, MAX_JAMO_RUN)))

/** 후원 메시지만 읽는다(최대 100자). 닉네임·금액은 읽지 않아 익명 후원자 실명이 소리로 새지 않는다. 메시지가 없으면 빈 문자열. */
export const ttsText = (d: Donation) => readableJamo(d.message?.trim().slice(0, MAX_MESSAGE) ?? '')

export const ttsSupported = () => typeof speechSynthesis !== 'undefined' && typeof SpeechSynthesisUtterance !== 'undefined'

export function speak(text: string, volume = 1) {
  if (!ttsSupported()) return
  speechSynthesis.cancel()
  const u = new SpeechSynthesisUtterance(text)
  u.lang = 'ko-KR'
  u.volume = volume
  speechSynthesis.speak(u)
}

export function stopSpeaking() {
  if (ttsSupported()) speechSynthesis.cancel()
}

let audio: AudioContext | undefined

/** 후원 알림 "띠링": 외부 파일 없이 두 음 사인파(약 0.5초). 자동재생 정책으로 막혀 있으면 조용히 건너뛴다. */
export function chime(volume = 1) {
  if (volume <= 0 || typeof AudioContext === 'undefined') return
  try {
    audio ??= new AudioContext()
    const ctx = audio
    if (ctx.state === 'suspended') void ctx.resume().catch(() => {})
    const t0 = ctx.currentTime
    for (const [freq, at] of [[1318.5, 0], [1760, 0.12]] as const) {
      const osc = ctx.createOscillator()
      const gain = ctx.createGain()
      osc.type = 'sine'
      osc.frequency.value = freq
      gain.gain.setValueAtTime(0.0001, t0 + at)
      gain.gain.exponentialRampToValueAtTime(0.25 * volume, t0 + at + 0.01)
      gain.gain.exponentialRampToValueAtTime(0.0001, t0 + at + 0.4)
      osc.connect(gain).connect(ctx.destination)
      osc.start(t0 + at)
      osc.stop(t0 + at + 0.42)
    }
  } catch {
    /* 오디오를 못 쓰는 환경: 소리 없이 알림만 */
  }
}

export function loadTtsOn() {
  try {
    return localStorage.getItem(KEY) !== 'off'
  } catch {
    return true
  }
}

export function saveTtsOn(on: boolean) {
  try {
    localStorage.setItem(KEY, on ? 'on' : 'off')
  } catch {
    /* 저장 실패해도 이번 방문에는 적용된다 */
  }
}
