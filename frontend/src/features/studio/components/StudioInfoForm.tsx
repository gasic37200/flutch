import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import type { Category } from '../../../api/generated/Category.ts'
import Button from '../../../components/ui/Button.tsx'
import { Chip } from '../../../components/ui/Chip.tsx'
import Field from '../../../components/ui/Field.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import { CATEGORIES, CATEGORY_LABEL } from '../../broadcast/categories.ts'
import { MAX_TAG_LENGTH, MAX_TAGS, useBroadcastForm } from '../../broadcast/hooks/useBroadcastForm.ts'

const TITLE_MAX = 50

// 방송 중 정보 수정(제목·카테고리·태그, features/02). 저장하면 시청자 화면에 바로 반영된다. 로직은 useBroadcastForm.
export default function StudioInfoForm({ broadcast, onSaved = () => {} }: { broadcast: Broadcast; onSaved?: () => void }) {
  const f = useBroadcastForm(broadcast, onSaved)
  const { values, set, error } = f
  return (
    <form onSubmit={f.submit} noValidate className='flex flex-col gap-3'>
      {error?.message && (
        <p role='alert' className='flex items-start gap-2 bg-surface-2 p-2.5 text-[13px]'>
          <Icon name='alert' className='mt-0.5 size-4 shrink-0 text-brand' />
          {error.message}
        </p>
      )}
      <Field label='제목' value={values.title} onChange={(e) => set.title(e.target.value)} maxLength={TITLE_MAX} error={error?.fields.title} required />
      <label className='flex flex-col gap-1 text-[13px] font-semibold text-text-muted'>
        카테고리
        <span className='chamfer flex bg-surface-2 [--c:6px]'>
          <select
            value={values.category}
            onChange={(e) => set.category(e.target.value as Category)}
            className='h-[42px] grow bg-transparent px-2.5 text-sm font-normal text-text outline-none'
          >
            {CATEGORIES.map((c) => (
              <option key={c} value={c} className='bg-surface-2'>
                {CATEGORY_LABEL[c]}
              </option>
            ))}
          </select>
        </span>
        {error?.fields.category && <span className='text-xs font-normal text-text'>{error.fields.category}</span>}
      </label>
      <div className='flex flex-col gap-1.5 text-[13px] font-semibold text-text-muted'>
        <label htmlFor='studio-tag'>태그 (최대 {MAX_TAGS}개)</label>
        <div className='flex flex-wrap items-center gap-1.5'>
          {values.tags.map((t) => (
            <Chip key={t} className='gap-1.5 px-2.5 py-1 text-[13px] font-normal text-text'>
              {t}
              <button type='button' onClick={() => f.removeTag(t)} aria-label={`${t} 태그 삭제`} className='text-text-muted hover:text-text'>
                ×
              </button>
            </Chip>
          ))}
          {values.tags.length < MAX_TAGS && (
            <input
              id='studio-tag'
              value={values.tagDraft}
              onChange={(e) => set.tagDraft(e.target.value)}
              onKeyDown={f.tagKeyDown}
              maxLength={MAX_TAG_LENGTH}
              placeholder='태그 입력 후 스페이스'
              className='h-7 min-w-0 grow basis-28 bg-transparent px-1 text-[13px] font-normal text-text outline-none placeholder:text-text-dim'
            />
          )}
        </div>
        {error?.fields.tags && <span className='text-xs font-normal text-text'>{error.fields.tags}</span>}
      </div>
      <Button type='submit' loading={f.pending}>
        변경 저장
      </Button>
    </form>
  )
}
