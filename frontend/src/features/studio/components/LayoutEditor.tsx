import type { CSSProperties, ReactNode } from 'react'
import type { OverlayLayout } from '../../../api/generated/OverlayLayout.ts'
import { alertStyle, camStyle, chatStyle } from '../../live/overlayLayout.ts'
import { AlertCard, ALERT_WIDTH } from '../../donation/components/DonationAlert.tsx'
import { centerX, type LayoutKey, type useLayoutEditor } from '../hooks/useLayoutEditor.ts'

type Editor = ReturnType<typeof useLayoutEditor>

const btn = 'chamfer h-9 px-3.5 text-[13px] [--c:6px] focus-visible:outline-2 focus-visible:outline-text'

/** 시안 StudioLayout: 미리보기 위 편집 도구 줄(기본 위치로·취소·완료) */
export function LayoutToolbar({ editor: e }: { editor: Editor }) {
  return (
    <div role='toolbar' aria-label='배치 편집' className='chamfer flex flex-wrap items-center gap-x-3 gap-y-2.5 bg-surface-2 px-3.5 py-2.5 [--c:6px]'>
      <b className='text-[15px] text-brand'>배치 편집</b>
      <span className='text-[13px] text-text-muted'>요소를 끌어서 옮기세요 · 선택 후 방향키 1%, Shift+방향키 5%</span>
      <span className='grow' />
      <button type='button' onClick={e.reset} className={`${btn} bg-border text-text`}>
        기본 위치로
      </button>
      <button type='button' onClick={e.cancel} className={`${btn} text-text-muted ring-1 ring-border ring-inset`}>
        취소
      </button>
      <button type='button' onClick={() => void e.done()} disabled={e.saving} className={`${btn} bg-brand px-4 font-bold text-bg disabled:opacity-60`}>
        완료
      </button>
    </div>
  )
}

const LABEL: Record<LayoutKey, string> = { alert: '후원 알림', cam: '웹캠', chat: '채팅 오버레이' }
const SAMPLE_CHAT: [string, string, string][] = [
  ['에임장인', 'ㄷㄷ 방금 그거 실화냐', 'text-ring-2'],
  ['밤샘냥', '와 미쳤다 ㄷㄷ', 'text-ring-4'],
  ['수다곰', '클러치각이다!', 'text-ring-5'],
]

/** 미리보기 위에 얹는 편집 층: 세 요소를 점선+라벨로 보여 주고 끌어서 옮긴다. 웹캠 영상은 아래 미리보기가 같은 초안 위치로 따라온다. */
export function LayoutLayer({ editor: { boxRef, handlers }, layout: l, selected: sel, hasCam }: { editor: Editor; layout: OverlayLayout; selected: LayoutKey; hasCam: boolean }) {
  const centered = (k: LayoutKey) => Math.abs(centerX(l, k) - 50) < 0.05
  const item = (key: LayoutKey, style: CSSProperties, cls: string, tag: { cls: string; text?: string }, body: ReactNode) => {
    const on = sel === key
    const pos = `가로 ${l[key].x}% · 세로 ${l[key].y}%`
    return (
      <div
        role='button'
        tabIndex={0}
        aria-label={`${LABEL[key]} 위치, ${pos}`}
        style={style}
        className={`absolute cursor-grab touch-none outline-offset-3 select-none active:cursor-grabbing ${on ? 'z-10 outline-2 outline-text' : 'outline-1 outline-brand outline-dashed'} ${cls}`}
        data-key={key}
        {...handlers}
      >
        <span className={`absolute px-1.5 py-0.5 font-sans text-[11px] font-bold whitespace-nowrap text-bg ${on ? 'bg-text' : 'bg-brand'} ${tag.cls}`}>
          {on ? `${tag.text ?? LABEL[key]} · ${pos}${centered(key) ? ' · 가운데 맞춤' : ''}` : (tag.text ?? LABEL[key])}
        </span>
        {body}
      </div>
    )
  }
  return (
    <div ref={boxRef} className='absolute inset-0 z-20'>
      {centered(sel) && <span aria-hidden className='absolute inset-y-0 left-1/2 border-l border-dashed border-brand/70' />}
      {item(
        'chat',
        chatStyle(l),
        'flex flex-col gap-1 text-[length:clamp(11px,1.2cqw,14px)] leading-[1.45]',
        { cls: '-top-6.5 -left-1', text: '채팅 오버레이 · 시청자 화면에 표시 (시청자가 끌 수 있음)' },
        SAMPLE_CHAT.map(([n, t, c]) => (
          <p key={n} className='w-fit bg-bg/60 px-2 py-[3px]'>
            <b className={c}>{n}</b> {t}
          </p>
        )),
      )}
      {item(
        'cam',
        camStyle(l),
        `flex aspect-video items-center justify-center text-[length:clamp(10px,1.2cqw,13px)] text-text-muted ${hasCam ? '' : 'bg-surface-2 ring-1 ring-brand'}`,
        { cls: '-top-6.5 -left-1' },
        !hasCam && '웹캠',
      )}
      {item(
        'alert',
        alertStyle(l),
        `-translate-x-1/2 ${ALERT_WIDTH}`,
        { cls: 'top-[calc(100%+8px)] left-1/2 -translate-x-1/2' },
        <AlertCard
          title={
            <>
              후원 알림 미리보기 <span className='font-mono text-brand'>5,000</span> Beat
            </>
          }
          message='실제 크기 · 일반 42% / 10,000 이상 50%'
        />,
      )}
    </div>
  )
}
