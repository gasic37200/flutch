import type { Donation } from '../../api/generated/Donation.ts'

const MAX_MESSAGE = 100
const KEY = 'flutch.donationTts'

/** 후원 메시지만 읽는다(최대 100자). 닉네임·금액은 읽지 않아 익명 후원자 실명이 소리로 새지 않는다. 메시지가 없으면 빈 문자열. */
export const ttsText = (d: Donation) => d.message?.trim().slice(0, MAX_MESSAGE) ?? ''

export const ttsSupported = () => typeof speechSynthesis !== 'undefined' && typeof SpeechSynthesisUtterance !== 'undefined'

export function speak(text: string) {
  if (!ttsSupported()) return
  speechSynthesis.cancel()
  const u = new SpeechSynthesisUtterance(text)
  u.lang = 'ko-KR'
  speechSynthesis.speak(u)
}

export function stopSpeaking() {
  if (ttsSupported()) speechSynthesis.cancel()
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
