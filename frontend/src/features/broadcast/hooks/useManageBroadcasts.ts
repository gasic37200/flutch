import { useState } from 'react'
import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import { useToast } from '../../../hooks/useToast.ts'
import { formError } from '../../auth/hooks/formError.ts'
import { useDeleteBroadcast, useMyBroadcasts } from './useBroadcasts.ts'

export const READY_LIMIT = 5
const ORDER = { READY: 0, LIVE: 1, ENDED: 2 }

// 끝난 방송은 종료 시각, 나머지는 등록 시각 기준 최신순. 서버는 등록(id)순이라 늦게 진행한 방송이 아래로 밀린다.
const when = (b: Broadcast) => Date.parse(b.endedAt ?? b.startedAt ?? b.createdAt)
export const recentFirst = (a: Broadcast, b: Broadcast) => when(b) - when(a)

/** 내 방송 관리: READY → LIVE → ENDED 순(같은 상태끼리는 최신순) + 삭제 확인 상태. */
export function useManageBroadcasts() {
  const query = useMyBroadcasts()
  const [target, setTarget] = useState<Broadcast | null>(null)
  const remove = useDeleteBroadcast()
  const toast = useToast()

  const items = [...(query.data?.items ?? [])].sort((a, b) => ORDER[a.status] - ORDER[b.status] || recentFirst(a, b))

  const confirmDelete = () => {
    if (!target) return
    remove.mutate(target.id, {
      onSuccess: () => toast({ tone: 'success', message: '방송을 삭제했어요.' }),
      onError: (e) => toast({ tone: 'error', message: formError(e, []).message ?? '삭제하지 못했어요.' }),
      onSettled: () => setTarget(null),
    })
  }

  return {
    query,
    items,
    readyCount: items.filter((b) => b.status === 'READY').length,
    target,
    askDelete: setTarget,
    cancelDelete: () => setTarget(null),
    confirmDelete,
    deleting: remove.isPending,
  }
}

/** 삭제 확인 문구. LIVE는 먼저 송출이 끝난다는 것을 알린다. */
export function deleteMessage(b: Broadcast) {
  if (b.status === 'LIVE') return '송출이 종료된 뒤 삭제돼요. 지금 진행 중인 방송이 바로 끝나요.'
  if (b.status === 'ENDED') return '목록에서 사라지고 녹화 파일도 지워져요. 되돌릴 수 없어요.'
  return '삭제하면 되돌릴 수 없어요.'
}

/** 스튜디오(송출 화면, T09/T10)로 가는 링크. 방송 id를 쿼리로 넘긴다. */
export const studioPath = (id: number) => `/studio?broadcast=${id}`
