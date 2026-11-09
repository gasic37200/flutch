import type { Broadcast } from '../../../api/generated/Broadcast.ts'
import Button from '../../../components/ui/Button.tsx'
import { ChoiceChip, Chip } from '../../../components/ui/Chip.tsx'
import Field from '../../../components/ui/Field.tsx'
import Icon from '../../../components/ui/Icon.tsx'
import Panel from '../../../components/ui/Panel.tsx'
import Switch from '../../../components/ui/Switch.tsx'
import { CATEGORIES, CATEGORY_LABEL } from '../categories.ts'
import { MAX_TAG_LENGTH, MAX_TAGS, useBroadcastForm } from '../hooks/useBroadcastForm.ts'

const lockedNote = '지금 상태에서는 바꿀 수 없어요.'

export default function BroadcastForm({ existing }: { existing?: Broadcast }) {
  const f = useBroadcastForm(existing)
  const { values, set, locked, error } = f
  return (
    <form onSubmit={f.submit} noValidate>
      <Panel innerClassName='space-y-5 p-5'>
        {!existing && (
          <div className='flex justify-end'>
            <Button variant='secondary' size='sm' onClick={f.copyPrevious} disabled={!f.canCopy}>
              이전 방송 정보 복사
            </Button>
          </div>
        )}

        <Field
          label='제목'
          value={values.title}
          onChange={(e) => set.title(e.target.value)}
          maxLength={50}
          counter
          placeholder='방송 제목 (2~50자)'
          error={error?.fields.title}
          required
        />

        <fieldset disabled={locked.category} className='min-w-0'>
          <legend className='mb-1 text-sm font-semibold text-text-muted'>카테고리</legend>
          <div className='flex flex-wrap gap-2'>
            {CATEGORIES.map((c) => (
              <ChoiceChip key={c} selected={values.category === c} onClick={() => set.category(c)}>
                {CATEGORY_LABEL[c]}
              </ChoiceChip>
            ))}
          </div>
          {(locked.category || error?.fields.category) && <p className='mt-1 text-xs text-text-dim'>{error?.fields.category ?? lockedNote}</p>}
        </fieldset>

        <div>
          <Field
            label={`태그 (최대 ${MAX_TAGS}개)`}
            value={values.tagDraft}
            onChange={(e) => set.tagDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ',') {
                e.preventDefault()
                f.addTag()
              }
            }}
            maxLength={MAX_TAG_LENGTH}
            placeholder={values.tags.length >= MAX_TAGS ? '태그는 3개까지예요' : '입력 후 Enter로 추가'}
            disabled={locked.tags || values.tags.length >= MAX_TAGS}
            hint={locked.tags ? lockedNote : '게임 이름은 태그로 넣어 주세요.'}
            error={error?.fields.tags}
          />
          {values.tags.length > 0 && (
            <ul className='mt-2 flex flex-wrap gap-2' aria-label='추가한 태그'>
              {values.tags.map((t) => (
                <li key={t} className='inline-flex items-center gap-1'>
                  <Chip>#{t}</Chip>
                  {!locked.tags && (
                    <button type='button' onClick={() => f.removeTag(t)} aria-label={`${t} 태그 삭제`} className='text-text-muted hover:text-text'>
                      <Icon name='close' className='size-4' />
                    </button>
                  )}
                </li>
              ))}
            </ul>
          )}
        </div>

        <div className='flex items-center justify-between gap-4'>
          <div>
            <p className='text-sm font-semibold'>방송 녹화</p>
            <p className='text-xs text-text-dim'>{locked.record ? '방송을 시작한 뒤에는 바꿀 수 없어요.' : '끝난 방송을 다시보기로 남겨요.'}</p>
          </div>
          <Switch label='방송 녹화' checked={values.record} onChange={set.record} disabled={locked.record} />
        </div>

        {error?.message && (
          <p role='alert' className='flex items-start gap-2 bg-surface-2 p-3 text-sm'>
            <Icon name='alert' className='mt-0.5 size-4 shrink-0 text-brand' />
            {error.message}
          </p>
        )}

        <div className='flex justify-end gap-2'>
          <Button variant='secondary' to='/manage'>
            취소
          </Button>
          <Button type='submit' loading={f.pending}>
            {existing ? '저장' : '등록'}
          </Button>
        </div>
      </Panel>
    </form>
  )
}
