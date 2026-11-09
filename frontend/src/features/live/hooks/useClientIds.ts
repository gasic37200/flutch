import { useState } from 'react'

const ID_RE = /^[A-Za-z0-9_-]{8,64}$/

function stableId(storage: () => Storage, key: string): string {
  try {
    const s = storage()
    const saved = s.getItem(key)
    if (saved && ID_RE.test(saved)) return saved
    const id = crypto.randomUUID()
    s.setItem(key, id)
    return id
  } catch {
    return crypto.randomUUID() // 저장소가 막힌 환경: 이번 화면 동안만 유지
  }
}

/** 브라우저 탭마다 하나(sessionStorage). 송출 탭과 시청 탭을 서버가 구분하는 데 쓴다. */
export function useTabId(): string {
  return useState(() => stableId(() => sessionStorage, 'flutch.tabId'))[0]
}

/** 브라우저마다 하나(localStorage). 비로그인 시청자 식별(`viewer-token`의 guestId). */
export function useGuestId(): string {
  return useState(() => stableId(() => localStorage, 'flutch.guestId'))[0]
}
