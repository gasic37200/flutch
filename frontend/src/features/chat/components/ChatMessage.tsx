import type { Chat } from '../../../api/generated/Chat.ts'
import Badge from '../../../components/ui/Badge.tsx'
import Button from '../../../components/ui/Button.tsx'
import ConfirmDialog from '../../../components/ui/ConfirmDialog.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import { profileTone } from '../../../components/ui/mascot.ts'
import ViewerPopup from '../../moderation/components/ViewerPopup.tsx'
import type { ChatApi } from '../hooks/useChat.ts'
import { MAX_CHAT } from '../hooks/useChatComposer.ts'
import { useChatItem } from '../hooks/useChatItem.ts'

const clock = (iso: string) => new Date(iso).toLocaleTimeString('ko-KR', { hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false })

const link = 'px-2 py-1 text-xs text-text-muted hover:text-text'

// 채팅 한 줄. 후원 강조 줄(T23)은 목록에서 이 컴포넌트 대신 별도 줄을 그리면 된다.
export default function ChatMessage({ chat, item, isOwner, live }: { chat: ChatApi; item: Chat; isOwner: boolean; live: boolean }) {
  const m = useChatItem(chat, item, { isOwner, live })
  const tone = profileTone(item.author.id)
  return (
    <li className='group relative'>
      <div className='flex flex-wrap items-baseline gap-x-1.5 gap-y-0.5'>
        {item.badges.includes('streamer') && <Badge tone='streamer' />}
        {item.badges.includes('donor') && <Badge tone='donor' />}
        <button
          type='button'
          aria-label={`${item.author.nickname}님 정보`}
          aria-haspopup='dialog'
          onClick={m.openPopup}
          className={`font-bold break-all hover:underline ${tone.nicknameClass}`}
        >
          {item.author.nickname}
        </button>
        {item.deleted ? (
          <span className='text-text-dim italic'>삭제된 메시지입니다</span>
        ) : m.editing ? null : (
          <span className='break-words whitespace-pre-wrap'>{item.body}</span>
        )}
        {item.edited && !item.deleted && !m.editing && <Badge tone='edited' />}
        <time dateTime={item.createdAt} className='text-[11px] text-text-dim opacity-0 group-focus-within:opacity-100 group-hover:opacity-100 pointer-coarse:hidden'>
          {clock(item.createdAt)}
        </time>
        {m.hasMenu && !m.editing && (
          <button
            type='button'
            aria-label={`${item.author.nickname}님의 메시지 메뉴`}
            aria-expanded={m.menu}
            onClick={m.toggleMenu}
            className='absolute top-0 right-0 flex size-5 items-center justify-center bg-surface text-text-muted opacity-0 group-focus-within:opacity-100 group-hover:opacity-100 pointer-coarse:opacity-100 aria-expanded:opacity-100 hover:text-text'
          >
            <Icon name='more' className='size-4' />
          </button>
        )}
      </div>

      {m.menu && (
        <div role='menu' className='mt-1 flex gap-1'>
          {m.canEdit && (
            <button type='button' role='menuitem' className={link} onClick={m.startEdit}>
              수정
            </button>
          )}
          {m.canDelete && (
            <button type='button' role='menuitem' className={link} onClick={m.askDelete}>
              삭제
            </button>
          )}
          {!m.canEdit && !m.canDelete && <span className='px-2 py-1 text-xs text-text-dim'>할 수 있는 작업이 없어요</span>}
        </div>
      )}

      {m.editing && (
        <div className='mt-1 flex flex-col gap-1.5'>
          <input
            autoFocus
            aria-label='채팅 수정'
            maxLength={MAX_CHAT}
            value={m.draft}
            onChange={(e) => m.setDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.nativeEvent.isComposing || e.keyCode === 229) return
              if (e.key === 'Enter') {
                e.preventDefault()
                m.saveEdit()
              } else if (e.key === 'Escape') m.cancelEdit()
            }}
            className='h-9 w-full bg-surface-2 px-2 text-sm text-text outline-1 outline-brand-line focus:outline-brand'
          />
          <div className='flex justify-end gap-1.5'>
            <Button size='sm' variant='neutral' onClick={m.cancelEdit} disabled={m.busy}>
              취소
            </Button>
            <Button size='sm' onClick={m.saveEdit} loading={m.busy} disabled={!m.draft.trim()}>
              저장
            </Button>
          </div>
        </div>
      )}

      {m.error && (
        <p role='alert' className='mt-1 flex items-center gap-1 text-xs text-text'>
          <Icon name='alert' className='size-3.5' />
          {m.error}
        </p>
      )}

      {m.popup && <ViewerPopup user={item.author} badges={item.badges} ownerId={m.ownerId} onClose={m.closePopup} />}

      <ConfirmDialog
        open={m.confirming}
        title='메시지를 삭제할까요?'
        message='삭제하면 "삭제된 메시지입니다"로 바뀌고 되돌릴 수 없어요.'
        confirmLabel='삭제'
        loading={m.busy}
        onConfirm={m.confirmDelete}
        onCancel={m.cancelDelete}
      />
    </li>
  )
}
