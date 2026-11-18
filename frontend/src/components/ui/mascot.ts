// 마스코트 표정 6종 × 링 색 6종. 사용자 id로 고정(DESIGN §5). 클래스 문자열은 Tailwind가 찾도록 전부 리터럴로 둔다.
export type Face = 'calm' | 'excited' | 'grumpy' | 'happy' | 'joy' | 'sleepy'

const faces: Face[] = ['calm', 'excited', 'grumpy', 'happy', 'joy', 'sleepy']
const rings = ['border-ring-1', 'border-ring-2', 'border-ring-3', 'border-ring-4', 'border-ring-5', 'border-ring-6']
// 채팅 닉네임 색도 같은 팔레트
const nicknames = ['text-ring-1', 'text-ring-2', 'text-ring-3', 'text-ring-4', 'text-ring-5', 'text-ring-6']

export const faceSrc = (face: Face) => `/brand/face-${face}.png`

// 상황 그림 = 이모티콘 16종 중 쓰는 것(투명 PNG, 원형·배경 덧씌우지 않음). 얼굴(face-*)은 프로필 사진 전용.
const emoteFiles = {
  sleepy: '05-sleepy', // 빈 화면
  tilt: '03-tilt', // 검색 결과 없음
  hiding: '15-hiding', // 404·권한 없음
  sweat: '14-sweat', // 500·에러
  crying: '09-crying', // 연결 끊김·탈퇴
  excited: '02-excited', // 결제·후원 완료
  bye: '13-bye', // 가입 완료·방송 종료
  humming: '16-humming', // 곧 시작
  surprised: '04-surprised', // SUPER CLUTCH
}
export type Emote = keyof typeof emoteFiles
export const emoteSrc = (emote: Emote) => `/brand/emotes/${emoteFiles[emote]}.png`

export function profileTone(userId: number) {
  const id = Math.abs(userId)
  const color = Math.floor(id / 6) % 6
  return { face: faces[id % 6], ringClass: rings[color], nicknameClass: nicknames[color] }
}
