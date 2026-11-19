import type { Donation } from '../../api/generated/Donation.ts'
import { beats, donorName } from './format.ts'

const MAX_MESSAGE = 100
const KEY = 'flutch.donationTts'

/** 익명은 소유자 이벤트에 실명이 있어도 공개 이름으로 읽는다(스튜디오 탭 소리가 방송에 실릴 수 있음). */
export function ttsText(d: Donation) {
  const who = d.anonymous ? '익명의 후원자' : donorName(d)
  const msg = d.message?.trim().slice(0, MAX_MESSAGE)
  return `${who}님, ${beats(d.amount)} Beat 후원.${msg ? ` ${msg}` : ''}`
}

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
