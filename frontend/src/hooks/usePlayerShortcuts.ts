import { useEffect, useRef } from 'react'

export type PlayerShortcuts = {
  togglePlay?: () => void
  toggleMute?: () => void
  toggleFullscreen?: () => void
  /** 초 단위 이동(다시보기만) */
  seek?: (sec: number) => void
  /** 볼륨 증감(0~1 기준) */
  volume?: (delta: number) => void
}

// 입력창·버튼·링크에 포커스가 있으면 그 요소의 키 동작을 그대로 둔다
const SKIP = 'input,textarea,select,button,a,[contenteditable]:not([contenteditable="false"])'

/** 플레이어 단축키: 스페이스 재생/정지, M 음소거, F 전체화면, ←/→ 5초 이동, ↑/↓ 볼륨 10%. 한글 자판에서도 되게 `code`로 본다. */
export function usePlayerShortcuts(actions: PlayerShortcuts) {
  const ref = useRef(actions)
  useEffect(() => {
    ref.current = actions
  })
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.defaultPrevented || e.ctrlKey || e.metaKey || e.altKey) return
      if (e.target instanceof Element && e.target.closest(SKIP)) return
      const a = ref.current
      const fn = {
        Space: a.togglePlay,
        KeyM: a.toggleMute,
        KeyF: a.toggleFullscreen,
        ArrowLeft: a.seek && (() => a.seek?.(-5)),
        ArrowRight: a.seek && (() => a.seek?.(5)),
        ArrowUp: a.volume && (() => a.volume?.(0.1)),
        ArrowDown: a.volume && (() => a.volume?.(-0.1)),
      }[e.code]
      if (!fn) return
      // 스페이스 페이지 스크롤·포커스된 video의 기본 키 동작과 겹치지 않게 막는다
      e.preventDefault()
      fn()
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])
}
