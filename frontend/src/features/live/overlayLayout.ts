import type { CSSProperties } from 'react'
import type { OverlayLayout } from '../../api/generated/OverlayLayout.ts'

/** 서버 값이 null일 때(시안 StudioLayout): 알림 위 가운데 4%, 웹캠 오른쪽 아래(right 2%·bottom 3%·폭 22%), 채팅 왼쪽 아래(left 2%·bottom 6%·폭 30%) */
export const DEFAULT_LAYOUT: OverlayLayout = {
  alert: { x: 50, y: 4 },
  cam: { x: 76, y: 75, w: 22 },
  chat: { x: 2, y: 94, w: 30 },
}

/** 시청 화면 아래 컨트롤 줄(h-12)을 가리지 않게 웹캠·채팅을 올리는 높이(CSS 길이). 전체화면은 시안 WatchFullscreen의 12% */
export const RESERVE = '3.5rem'
export const FS_RESERVE = '12%'

export const layoutOf = (l: OverlayLayout | null | undefined) => l ?? DEFAULT_LAYOUT

// 위치는 영상 기준 %라 인라인 스타일로 둔다. 알림=가로 가운데·위 끝, 웹캠=왼쪽 위(16:9라 높이% = 폭%), 채팅=왼쪽·아래 끝(위로 쌓이고 영상 위로 넘치지 않게 max-height)
// `reserve`: 시청 화면은 아래 컨트롤 줄을 가리지 않게 웹캠·채팅을 그 높이만큼 위로 올린다.
export const alertStyle = ({ alert: a }: OverlayLayout): CSSProperties => ({ left: `${a.x}%`, top: `${a.y}%` })
export const camStyle = ({ cam: c }: OverlayLayout, reserve = '0px'): CSSProperties => ({ left: `${c.x}%`, top: `max(0px, min(${c.y}%, calc(${100 - c.w}% - ${reserve})))`, width: `${c.w}%` })
export const chatStyle = ({ chat: c }: OverlayLayout, reserve = '0px'): CSSProperties => ({
  left: `${c.x}%`,
  bottom: `max(${100 - c.y}%, ${reserve})`,
  width: `${c.w}%`,
  maxHeight: `min(${c.y}%, calc(100% - ${reserve}))`,
})
