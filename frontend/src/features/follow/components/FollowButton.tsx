import { useQuery } from '@tanstack/react-query'
import { useLocation } from 'react-router'
import { api } from '../../../api/client.ts'
import type { UserProfile } from '../../../api/generated/UserProfile.ts'
import Button from '../../../components/ui/Button.tsx'
import { useAuth } from '../../auth/hooks/useAuth.ts'
import { useToggleFollow } from '../hooks/useFollow.ts'

/** 팔로우/언팔로우. 비로그인은 로그인으로 안내, 본인에게는 그리지 않는다. 상태는 서버가 준 프로필(`['user', id]` 캐시 공유). */
export default function FollowButton({ userId, size }: { userId: number; size?: 'md' | 'lg' }) {
  const { me } = useAuth()
  const { pathname, search } = useLocation()
  const toggle = useToggleFollow(userId)
  const profile = useQuery({ queryKey: ['user', userId], queryFn: () => api<UserProfile>(`/users/${userId}`), enabled: me !== null && me.id !== userId })
  const following = profile.data?.following ?? false
  if (me?.id === userId) return null
  if (me === null) {
    return (
      <Button variant='secondary' size={size} to={`/login?next=${encodeURIComponent(pathname + search)}`}>
        팔로우
      </Button>
    )
  }
  return (
    <Button
      variant={following ? 'neutral' : 'primary'}
      size={size}
      loading={toggle.isPending || profile.isPending}
      aria-pressed={following}
      onClick={() => toggle.mutate(!following)}
    >
      {following ? '팔로잉' : '팔로우'}
    </Button>
  )
}
