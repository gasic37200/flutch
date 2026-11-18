import { useToggleFollow } from '../hooks/useFollow.ts'

/** PULSE 오프라인 행의 "팔로우 취소"(시안: 회색 테두리 보조 버튼). 성공하면 PULSE 캐시가 갱신돼 행이 사라진다. */
export default function UnfollowButton({ userId, nickname }: { userId: number; nickname: string }) {
  const toggle = useToggleFollow(userId)
  return (
    <button
      type='button'
      aria-label={`${nickname} 팔로우 취소`}
      disabled={toggle.isPending}
      onClick={() => toggle.mutate(false)}
      className='chamfer h-9 bg-border p-px text-[13px] text-text-muted [--c:6px] hover:text-text disabled:opacity-60'
    >
      <span className='chamfer-inner flex h-full items-center bg-surface px-[11px]'>팔로우 취소</span>
    </button>
  )
}
