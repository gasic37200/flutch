import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import Badge from '../../../components/ui/Badge.tsx'
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
const TITLE_MAX = 50

type Props = { heading: string; owner: { id: number; nickname: string }; existing?: Broadcast }

export default function BroadcastForm({ heading, owner, existing }: Props) {
  const f = useBroadcastForm(existing)
  const { values, set, locked, error } = f
  const tags = values.tags.map((t) => `#${t}`).join(' ')
  const tagNote = error?.fields.tags ?? (locked.tags ? lockedNote : values.tagDraft.length >= MAX_TAG_LENGTH ? `태그는 ${MAX_TAG_LENGTH}자까지예요.` : '')

  return (
    // 폼 최대 640 + 미리보기 카드 300 고정: 넓은 화면에서도 폭이 변하지 않는다
    <div className='mx-auto flex max-w-[968px] flex-wrap gap-7'>
      <form onSubmit={f.submit} noValidate className='flex min-w-0 max-w-[640px] grow basis-[480px] flex-col gap-5'>
        <div className='flex flex-wrap items-center gap-x-3 gap-y-2'>
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

        {(f.full || error?.message) && (
          <p role='alert' className='flex items-start gap-2 bg-surface-2 p-3 text-sm'>
            <Icon name='alert' className='mt-0.5 size-4 shrink-0 text-brand' />
            {f.full ? `준비 중인 방송이 ${READY_LIMIT}개라 더 등록할 수 없어요. 내 방송 관리에서 시작하거나 삭제해 주세요.` : error?.message}
          </p>
        )}

        <Field
          label='제목'
          value={values.title}
          onChange={(e) => set.title(e.target.value)}
          maxLength={TITLE_MAX}
          counter
          placeholder='방송 제목 (필수, 2~50자)'
          error={error?.fields.title}
          hint={values.title.length >= TITLE_MAX ? `제목은 ${TITLE_MAX}자까지예요.` : undefined}
          required
        />

        <fieldset disabled={locked.category} className='min-w-0'>
          <legend className='mb-1 text-sm font-semibold text-text-muted'>카테고리</legend>
          <div className='grid grid-cols-2 gap-2 sm:grid-cols-4'>
            {CATEGORIES.map((c) => {
              const on = values.category === c
              return (
                <span key={c} className='inline-flex has-[:focus-visible]:drop-shadow-focus'>
                  <button
                    type='button'
                    aria-pressed={on}
                    onClick={() => set.category(c)}
                    className={`chamfer h-10 grow text-sm [--c:6px] disabled:cursor-not-allowed disabled:opacity-60 ${on ? 'bg-brand font-bold text-bg' : 'bg-surface-2 text-text-muted hover:text-text'}`}
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
            태그 <span className='font-normal text-text-dim'>(최대 {MAX_TAGS}개, 태그당 {MAX_TAG_LENGTH}자 · 스페이스나 Enter로 확정 · 게임 이름은 태그로)</span>
          </label>
          <div className='focus-within:drop-shadow-field'>
            <div className={`chamfer p-px [--c:6px] ${locked.tags ? 'bg-surface-dim' : 'bg-brand-line focus-within:bg-brand'}`}>
              <div className='chamfer-inner flex min-h-10 flex-wrap items-center gap-1.5 bg-surface px-2.5 py-1'>
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
                {/* #과 입력칸을 한 묶음으로: 좁은 폭에서 두 줄로 갈라지지 않게 */}
                <span className='flex min-w-[120px] grow items-center gap-1'>
                  {!locked.tags && values.tags.length < MAX_TAGS && (
                    <span aria-hidden className='shrink-0 text-sm text-text-dim'>
                      #
                    </span>
                  )}
                  <input
                    id='broadcast-tag'
                    value={values.tagDraft}
                    onChange={(e) => set.tagDraft(e.target.value)}
                    onKeyDown={f.tagKeyDown}
                    maxLength={MAX_TAG_LENGTH}
                    placeholder={locked.tags ? '끝난 방송은 태그를 바꿀 수 없어요' : values.tags.length >= MAX_TAGS ? '태그는 3개까지예요' : '태그 입력 후 스페이스'}
                    disabled={locked.tags || values.tags.length >= MAX_TAGS}
                    aria-describedby='broadcast-tag-note'
                    className='min-w-0 grow bg-transparent text-sm text-text outline-none placeholder:text-text-dim disabled:cursor-not-allowed'
                  />
                </span>
              </div>
            </div>
          </div>
          <p id='broadcast-tag-note' className={`mt-1 min-h-4 text-xs ${error?.fields.tags ? 'text-text' : 'text-text-dim'}`}>
            {tagNote}
          </p>
        </div>

        <Panel innerClassName='flex items-center gap-3 px-4 py-3.5'>
          <div className='grow'>
            <p className='text-[15px] font-bold'>녹화해서 다시보기로 저장</p>
            <p className='text-xs text-text-dim'>{locked.record ? '방송을 시작한 뒤에는 바꿀 수 없어요.' : '최대 3시간 · 30일 보관 · 방송 시작 후에는 바꿀 수 없어요'}</p>
          </div>
          <Switch label='방송 녹화' checked={values.record} onChange={set.record} disabled={locked.record} />
        </Panel>

        {/* 주 버튼 + 보조 버튼 + 취소: 같은 높이, 좁으면 세로로 쌓아 폭을 맞춘다 */}
        <div className='flex flex-col gap-2.5 sm:flex-row sm:flex-wrap'>
          <Button type='submit' loading={f.pending} disabled={f.full} className='sm:min-w-44 [&>*]:grow'>
            {existing ? '저장' : '등록하고 스튜디오로'}
          </Button>
          {!existing && (
            <Button variant='secondary' onClick={f.saveToManage} disabled={f.pending || f.full} className='sm:min-w-44'>
              대기 방송으로 저장
            </Button>
          )}
          <Button variant='neutral' to='/manage' className='sm:min-w-24'>
            취소
          </Button>
        </div>
        <p className='text-[13px] text-text-dim'>제목·카테고리·태그는 방송 중에도 바꿀 수 있어요. 끝난 방송은 제목만 바꿀 수 있어요.</p>
      </form>

      <aside className='flex w-[300px] max-w-full min-w-0 flex-col gap-2.5 lg:sticky lg:top-24 lg:self-start'>
        <span className='text-[13px] text-text-muted'>목록에 이렇게 보여요</span>
        <div className='relative'>
          <LiveCard
            title={values.title.trim() || '방송 제목'}
            streamerId={owner.id}
            streamerName={tags ? `${owner.nickname} · ${tags}` : owner.nickname}
            category={CATEGORY_LABEL[values.category]}
            thumbnailUrl={existing?.thumbnailUrl}
            live={f.status === 'LIVE'}
          />
          {f.status !== 'LIVE' && <Badge tone={f.status === 'ENDED' ? 'ended' : 'ready'} className='absolute top-2 left-2' />}
        </div>
        <span className='text-xs text-text-dim'>썸네일은 방송 중 주기적으로 자동 캡처돼요.</span>
      </aside>
    </div>
  )
}
