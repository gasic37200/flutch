import { useRef, useState, type FocusEvent, type KeyboardEvent, type PointerEvent } from 'react'
import type { OverlayLayout } from '../../../api/generated/OverlayLayout.ts'
import { DEFAULT_LAYOUT } from '../../live/overlayLayout.ts'

export type LayoutKey = 'alert' | 'cam' | 'chat'
type Rect = { left: number; top: number; right: number; bottom: number; width: number; height: number }
const SNAP = 0.01 // 가운데 맞춤으로 붙는 거리(영상 폭 대비)

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v))
const round = (v: number) => clamp(Math.round(v * 10) / 10, 0, 100)

/** 요소의 가로 가운데(%) — 알림은 x가 가운데, 웹캠·채팅은 왼쪽 + 폭/2 */
export function centerX(l: OverlayLayout, key: LayoutKey) {
  return key === 'alert' ? l.alert.x : l[key].x + l[key].w / 2
}

/**
 * 스튜디오 "배치 편집"(시안 StudioLayout): 끌기·방향키(1%, Shift 5%)로 옮기고 완료하면 저장한다.
 * 위치는 영상 기준 %. 옮긴 뒤 요소의 화면상 사각형이 영상 밖으로 나가지 않게 픽셀로 잘라서 %로 되돌린다.
 */
export function useLayoutEditor(saved: OverlayLayout, save: (l: OverlayLayout) => Promise<unknown>) {
  const [draft, setDraft] = useState<OverlayLayout | null>(null) // null = 편집 중 아님
  const [selected, setSelected] = useState<LayoutKey>('alert')
  const [saving, setSaving] = useState(false)
  const boxRef = useRef<HTMLDivElement>(null)
  const drag = useRef<{ key: LayoutKey; x: number; y: number; start: OverlayLayout; el: Rect; box: Rect } | null>(null)

  /** start에서 key 요소를 (dx, dy)px 옮긴 배치. 영상 밖으로는 못 나가고, snap이면 가운데 근처에서 가운데로 붙는다. */
  const moved = (key: LayoutKey, start: OverlayLayout, el: Rect, box: Rect, dx: number, dy: number, snap: boolean): OverlayLayout => {
    dx = clamp(dx, box.left - el.left, box.right - el.right)
    dy = clamp(dy, box.top - el.top, box.bottom - el.bottom)
    if (snap) {
      const off = el.left + dx + el.width / 2 - (box.left + box.width / 2)
      if (Math.abs(off) < box.width * SNAP) dx -= off
    }
    const p = start[key]
    return { ...start, [key]: { ...p, x: round(p.x + (dx / box.width) * 100), y: round(p.y + (dy / box.height) * 100) } }
  }

  // 요소마다 data-key로 구분한다(렌더 중에 핸들러를 만드는 함수를 부르지 않게)
  const keyOf = (e: { currentTarget: HTMLElement }) => e.currentTarget.dataset.key as LayoutKey
  const handlers = {
    onPointerDown: (e: PointerEvent<HTMLElement>) => {
      const key = keyOf(e)
      const box = boxRef.current?.getBoundingClientRect()
      if (!draft || !box || e.button !== 0) return
      e.currentTarget.setPointerCapture(e.pointerId)
      e.currentTarget.focus()
      setSelected(key)
      drag.current = { key, x: e.clientX, y: e.clientY, start: draft, el: e.currentTarget.getBoundingClientRect(), box }
    },
    onPointerMove: (e: PointerEvent<HTMLElement>) => {
      const d = drag.current
      const key = keyOf(e)
      if (d?.key === key) setDraft(moved(key, d.start, d.el, d.box, e.clientX - d.x, e.clientY - d.y, true))
    },
    onPointerUp: () => void (drag.current = null),
    onPointerCancel: () => void (drag.current = null),
    onFocus: (e: FocusEvent<HTMLElement>) => setSelected(keyOf(e)),
    onKeyDown: (e: KeyboardEvent<HTMLElement>) => {
      const key = keyOf(e)
      const dir = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] }[e.key]
      const box = boxRef.current?.getBoundingClientRect()
      if (!dir || !draft || !box) return
      e.preventDefault()
      const step = e.shiftKey ? 5 : 1
      setDraft(moved(key, draft, e.currentTarget.getBoundingClientRect(), box, (dir[0] * step * box.width) / 100, (dir[1] * step * box.height) / 100, false))
    },
  }

  const done = async () => {
    if (!draft) return
    setSaving(true)
    try {
      await save(draft)
      setDraft(null)
    } catch {
      // 실패 안내는 save(호출자)가 한다. 편집 상태를 남겨 다시 시도할 수 있게 한다.
    } finally {
      setSaving(false)
    }
  }

  return {
    editing: draft !== null,
    /** 편집 중엔 초안, 아니면 저장된 배치 */
    layout: draft ?? saved,
    selected,
    saving,
    /** 편집 층(영상 영역)에 붙인다 */
    boxRef,
    /** 세 요소에 펼쳐 붙이고 data-key로 어느 요소인지 알린다 */
    handlers,
    start: () => setDraft(saved),
    cancel: () => setDraft(null),
    reset: () => setDraft(DEFAULT_LAYOUT),
    done,
  }
}
