import { faceSrc, profileTone } from './mascot.ts'

const sizes = { xs: 'size-5', sm: 'size-9', md: 'size-10', lg: 'size-16' }

type Props = { userId: number; nickname: string; src?: string | null; size?: keyof typeof sizes; className?: string }

// 업로드 이미지가 없으면 사용자 id로 고정된 마스코트 표정 + 링 색
export default function Avatar({ userId, nickname, src, size = 'md', className = '' }: Props) {
  const tone = profileTone(userId)
  return (
    <img
      src={src ?? faceSrc(tone.face)}
      alt={`${nickname} 프로필`}
      className={`shrink-0 rounded-full border-2 bg-surface-2 object-cover ${sizes[size]} ${tone.ringClass} ${className}`}
    />
  )
}
