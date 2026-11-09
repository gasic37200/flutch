// 마스코트 표정 6종 × 링 색 6종. 사용자 id로 고정(DESIGN §5). 클래스 문자열은 Tailwind가 찾도록 전부 리터럴로 둔다.
export type Face = 'calm' | 'excited' | 'grumpy' | 'happy' | 'joy' | 'sleepy'

const faces: Face[] = ['calm', 'excited', 'grumpy', 'happy', 'joy', 'sleepy']
const rings = ['border-ring-1', 'border-ring-2', 'border-ring-3', 'border-ring-4', 'border-ring-5', 'border-ring-6']
// 채팅 닉네임 색도 같은 팔레트
const nicknames = ['text-ring-1', 'text-ring-2', 'text-ring-3', 'text-ring-4', 'text-ring-5', 'text-ring-6']

export const faceSrc = (face: Face) => `/brand/face-${face}.png`

export function profileTone(userId: number) {
  const id = Math.abs(userId)
  const color = Math.floor(id / 6) % 6
  return { face: faces[id % 6], ringClass: rings[color], nicknameClass: nicknames[color] }
}
