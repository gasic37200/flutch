import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import Button from '../../../components/ui/Button.tsx'
import { Chip } from '../../../components/ui/Chip.tsx'
import Field from '../../../components/ui/Field.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import LiveCard from '../../../components/ui/LiveCard.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import Switch from '../../../components/ui/Switch.tsx'
import { CATEGORIES, CATEGORY_LABEL } from '../categories.ts'
import { MAX_TAG_LENGTH, MAX_TAGS, useBroadcastForm } from '../hooks/useBroadcastForm.ts'
import { READY_LIMIT } from '../hooks/useManageBroadcasts.ts'

const lockedNote = '지금 상태에서는 바꿀 수 없어요.'

type Props = { heading: string; owner: { id: number; nickname: string }; existing?: Broadcast }

export default function BroadcastForm({ heading, owner, existing }: Props) {
  const f = useBroadcastForm(existing)
  const { values, set, locked, error } = f
  const tags = values.tags.map((t) => `#${t}`).join(' ')

  return (
    <div className='flex max-w-[1080px] flex-wrap gap-7'>
      <form onSubmit={f.submit} noValidate className='flex min-w-0 grow-[999] basis-[520px] flex-col gap-5'>
        <div className='flex flex-wrap items-center gap-3'>
          <h1 className='text-2xl font-black sm:text-[26px]'>{heading}</h1>
          {!existing && (
            <>
              <span className='text-[13px] text-text-muted'>
                대기 중인 방송{' '}
                <b className='font-mono text-text'>
                  {f.readyCount}
                </b>{' '}
                / {READY_LIMIT}
              </span>
              <Button variant='neutral' className='ml-auto' onClick={f.copyPrevious} disabled={!f.canCopy}>
                <Icon name='copy' className='size-4' />
                이전 방송 정보 복사
              </Button>
            </>
          )}
        </div>

        <Field
          label='제목'
          value={values.title}
          onChange={(e) => set.title(e.target.value)}
          maxLength={50}
          counter
          placeholder='방송 제목 (필수, 2~50자)'
          error={error?.fields.title}
          required
        />

        <fieldset disabled={locked.category} className='min-w-0'>
          <legend className='mb-2 text-sm font-semibold text-text-muted'>카테고리</legend>
          <div className='grid grid-cols-2 gap-2 sm:grid-cols-4'>
            {CATEGORIES.map((c) => {
              const on = values.category === c
              return (
                <span key={c} className='inline-flex has-[:focus-visible]:drop-shadow-focus'>
                  <button
                    type='button'
                    aria-pressed={on}
                    onClick={() => set.category(c)}
                    className={`chamfer h-12 grow text-sm [--c:6px] disabled:cursor-not-allowed disabled:opacity-60 ${on ? 'bg-brand font-bold text-bg' : 'bg-surface-2 text-text-muted hover:text-text'}`}
                  >
                    {CATEGORY_LABEL[c]}
                  </button>
                </span>
              )
            })}
          </div>
          {(locked.category || error?.fields.category) && <p className='mt-1 text-xs text-text-dim'>{error?.fields.category ?? lockedNote}</p>}
        </fieldset>

        <div>
          <label htmlFor='broadcast-tag' className='mb-1 block text-sm font-semibold text-text-muted'>
            태그 <span className='font-normal text-text-dim'>(최대 {MAX_TAGS}개, 태그당 {MAX_TAG_LENGTH}자 · 게임 이름은 태그로)</span>
          </label>
          <div className='focus-within:drop-shadow-field'>
            <div className={`chamfer p-px [--c:6px] ${locked.tags ? 'bg-surface-dim' : 'bg-brand-line focus-within:bg-brand'}`}>
              <div className='chamfer-inner flex min-h-12 flex-wrap items-center gap-1.5 bg-surface px-2.5 py-1.5'>
                {values.tags.map((t) => (
                  <Chip key={t} className='gap-1 py-1 pr-1.5 text-[13px] text-text'>
                    #{t}
                    {!locked.tags && (
                      <button type='button' onClick={() => f.removeTag(t)} aria-label={`${t} 태그 삭제`} className='text-text-muted hover:text-text'>
                        <Icon name='close' className='size-3.5' />
                      </button>
                    )}
                  </Chip>
                ))}
                <input
                  id='broadcast-tag'
                  value={values.tagDraft}
                  onChange={(e) => set.tagDraft(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' || e.key === ',') {
                      e.preventDefault()
                      f.addTag()
                    }
                  }}
                  maxLength={MAX_TAG_LENGTH}
                  placeholder={values.tags.length >= MAX_TAGS ? '태그는 3개까지예요' : '태그 입력 후 Enter'}
                  disabled={locked.tags || values.tags.length >= MAX_TAGS}
                  aria-describedby='broadcast-tag-note'
                  className='min-w-[120px] grow bg-transparent text-sm text-text outline-none placeholder:text-text-dim disabled:cursor-not-allowed'
                />
              </div>
            </div>
          </div>
          <p id='broadcast-tag-note' className={`mt-1 text-xs ${error?.fields.tags ? 'text-text' : 'text-text-dim'}`}>
            {error?.fields.tags ?? (locked.tags ? lockedNote : '')}
          </p>
        </div>

        <Panel innerClassName='flex items-center gap-3 px-4 py-3.5'>
          <div className='grow'>
            <p className='text-[15px] font-bold'>녹화해서 다시보기로 저장</p>
            <p className='text-xs text-text-dim'>{locked.record ? '방송을 시작한 뒤에는 바꿀 수 없어요.' : '최대 3시간 · 30일 보관 · 방송 시작 후에는 바꿀 수 없어요'}</p>
          </div>
          <Switch label='방송 녹화' checked={values.record} onChange={set.record} disabled={locked.record} />
        </Panel>

        {error?.message && (
          <p role='alert' className='flex items-start gap-2 bg-surface-2 p-3 text-sm'>
            <Icon name='alert' className='mt-0.5 size-4 shrink-0 text-brand' />
            {error.message}
          </p>
        )}

        <div className='flex flex-wrap gap-2.5'>
          <Button type='submit' size='lg' loading={f.pending} className='min-w-40'>
            {existing ? '저장' : '등록'}
          </Button>
          <Button variant='neutral' size='lg' to='/manage'>
            취소
          </Button>
        </div>
        <p className='text-[13px] text-text-dim'>제목·카테고리·태그는 방송 중에도 바꿀 수 있어요. 끝난 방송은 제목만 바꿀 수 있어요.</p>
      </form>

      <aside className='flex min-w-0 basis-[300px] grow flex-col gap-2.5'>
        <span className='text-[13px] text-text-muted'>목록에 이렇게 보여요</span>
        <LiveCard
          to='/manage'
          title={values.title.trim() || '방송 제목'}
          streamerId={owner.id}
          streamerName={tags ? `${owner.nickname} · ${tags}` : owner.nickname}
          category={CATEGORY_LABEL[values.category]}
          thumbnailUrl={existing?.thumbnailUrl}
          live
        />
        <span className='text-xs text-text-dim'>썸네일은 방송 중 주기적으로 자동 캡처돼요.</span>
      </aside>
    </div>
  )
}
