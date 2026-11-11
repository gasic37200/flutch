import { useState } from 'react'
import { kstStamp } from '../../../lib/kst.ts'
import type { Badge as BadgeKind } from '../../../api/generated/Badge.ts'
import type { UserSummary } from '../../../api/generated/UserSummary.ts'
import Avatar from '../../../components/ui/Avatar.tsx'
import Badge from '../../../components/ui/Badge.tsx'
import Button from '../../../components/ui/Button.tsx'
import ConfirmDialog from '../../../components/ui/ConfirmDialog.tsx'
import Modal from '../../../components/ui/Modal.tsx'
import { profileTone } from '../../../components/ui/mascot.ts'
import { TIMEOUTS, useBanActions, useViewerChats } from '../hooks/useModeration.ts'


type Props = {
  user: UserSummary
  badges: BadgeKind[]
  /** 방송 소유자일 때만(그 사용자 id). 있으면 차단·채팅 기록 메뉴가 보인다. 서버가 다시 검사한다. */
  ownerId?: number
  onClose: () => void
}

// 채팅 닉네임을 눌렀을 때 뜨는 창. 시청자·비로그인은 프로필만, 방송 소유자는 차단·채팅 기록도.
export default function ViewerPopup({ user, badges, ownerId, onClose }: Props) {
  const [confirming, setConfirming] = useState(false)
  const [history, setHistory] = useState(false)
  const manage = ownerId !== undefined && ownerId !== user.id
  const actions = useBanActions(ownerId ?? 0)
  const chats = useViewerChats(ownerId ?? 0, user.id, manage && history)

  const act = async (f: () => Promise<boolean>) => {
    if (await f()) onClose()
  }

  return (
    <>
      <Modal open={!confirming} onClose={onClose} title='시청자 정보'>
        <div className='flex flex-col gap-3'>
          <div className='flex items-center gap-2.5'>
            <Avatar userId={user.id} nickname={user.nickname} size='sm' />
            <b className={`min-w-0 break-all ${profileTone(user.id).nicknameClass}`}>{user.nickname}</b>
            {badges.includes('streamer') && <Badge tone='streamer' />}
            {badges.includes('donor') && <Badge tone='donor' />}
          </div>

          {manage ? (
            <>
              <div className='flex flex-col gap-1.5'>
                <span className='text-xs text-text-muted'>임시차단</span>
                <div className='grid grid-cols-3 gap-1.5'>
                  {TIMEOUTS.map((t) => (
                    <Button key={t.duration} variant='neutral' size='sm' disabled={actions.busy} onClick={() => void act(() => actions.ban(user, t.duration))}>
                      {t.label}
                    </Button>
                  ))}
                </div>
              </div>
              <Button variant='outline' disabled={actions.busy} onClick={() => setConfirming(true)}>
                영구 밴
              </Button>
              <Button variant='neutral' size='sm' disabled={actions.busy} onClick={() => void act(() => actions.unban(user))}>
                차단 해제
              </Button>
              <Button variant='secondary' aria-expanded={history} onClick={() => setHistory((h) => !h)}>
                이 시청자의 채팅 기록
              </Button>
              {history && (
                <div role='region' aria-label='이 시청자의 채팅 기록' className='flex max-h-60 flex-col gap-2 overflow-y-auto bg-surface-dim p-3'>
                  {chats.query.isError ? (
                    <p className='text-text-muted'>채팅 기록을 불러오지 못했어요.</p>
                  ) : chats.query.isPending ? (
                    <p className='text-text-muted'>불러오는 중…</p>
                  ) : (
                    <>
                      {chats.query.hasNextPage && (
                        <Button size='sm' variant='neutral' onClick={() => void chats.query.fetchNextPage()} loading={chats.query.isFetchingNextPage}>
                          이전 기록 더 보기
                        </Button>
                      )}
                      {chats.items.length === 0 ? (
                        <p className='text-text-muted'>내 방송에서 쓴 채팅이 없어요.</p>
                      ) : (
                        <ul className='flex flex-col gap-1.5'>
                          {chats.items.map((c) => (
                            <li key={c.id} className='flex gap-2 text-[13px]'>
                              <time dateTime={c.createdAt} className='shrink-0 font-mono text-[11px] text-text-dim'>
                                {kstStamp(c.createdAt)}
                              </time>
                              {c.deleted ? <span className='text-text-dim italic'>삭제된 메시지입니다</span> : <span className='break-words'>{c.body}</span>}
                            </li>
                          ))}
                        </ul>
                      )}
                    </>
                  )}
                </div>
              )}
            </>
          ) : (
            badges.includes('streamer') && (
              <Button variant='secondary' to={`/users/${user.id}`} onClick={onClose}>
                스트리머 페이지 보기
              </Button>
            )
          )}
        </div>
      </Modal>
      <ConfirmDialog
        open={confirming}
        title={`${user.nickname}님을 영구 밴할까요?`}
        message='해제할 때까지 내 모든 방송에서 채팅·후원을 할 수 없고, 지금 방송의 채팅은 "삭제된 메시지"가 돼요.'
        confirmLabel='영구 밴'
        loading={actions.busy}
        onConfirm={() => void act(() => actions.ban(user, 'permanent'))}
        onCancel={() => setConfirming(false)}
      />
    </>
  )
}
