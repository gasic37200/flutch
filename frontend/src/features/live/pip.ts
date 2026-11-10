import type { PipCorner } from '../../api/generated/PipCorner.ts'

export const PIP_CORNERS: { value: PipCorner; label: string }[] = [
  { value: 'top_left', label: '왼쪽 위' },
  { value: 'top_right', label: '오른쪽 위' },
  { value: 'bottom_left', label: '왼쪽 아래' },
  { value: 'bottom_right', label: '오른쪽 아래' },
]

/** 카메라 PIP 위치 클래스. `y`는 위(top)·아래(bottom) 간격: 시청 화면은 컨트롤 바(h-12)와 LIVE 배지를 피한다. */
export function pipPosition(corner: PipCorner, y: { top: string; bottom: string }) {
  const [v, h] = corner.split('_') as ['top' | 'bottom', 'left' | 'right']
  return `${v === 'top' ? y.top : y.bottom} ${h === 'left' ? 'left-3' : 'right-3'}`
}
